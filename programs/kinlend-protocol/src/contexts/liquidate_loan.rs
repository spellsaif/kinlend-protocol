use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer};

use crate::state::{
    CollateralVaultState, LoanRegistryState, LoanRequestState, ProtocolVaultState,
};
use crate::errors::ErrorCode;

#[derive(Accounts)]
#[instruction(loan_id: u64)]
pub struct LiquidateLoan<'info> {
    /// The lender initiating liquidation. Must match the lender in the loan request.
    #[account(mut)]
    pub lender: Signer<'info>,

    /// Loan Request account (PDA) that holds the loan details.
    /// When closed, all lamports (including its rent deposit) are sent to the lender.
    #[account(
        mut,
        close = lender,
        seeds = [b"loan_request", loan_request.borrower.as_ref(), &loan_id.to_le_bytes()],
        bump
    )]
    pub loan_request: Box<Account<'info, LoanRequestState>>,

    /// Collateral Vault account (PDA) holding the collateral (in SOL).
    /// It is marked with `close = lender` so that any lamports left after fee transfer are returned to the lender.
    #[account(
        mut,
        close = lender,
        seeds = [b"collateral_vault", loan_request.key().as_ref()],
        bump = collateral_vault.bump
    )]
    pub collateral_vault: Box<Account<'info, CollateralVaultState>>,

    /// Loan Registry account (for tracking loans). Not used in distribution here.
    #[account(
        mut,
        seeds = [b"loan_registry"],
        bump
    )]
    pub loan_registry: Box<Account<'info, LoanRegistryState>>,

    /// Protocol Vault account (for protocol fees).
    #[account(
        mut,
        seeds = [b"protocol_vault"],
        bump = protocol_vault.bump
    )]
    pub protocol_vault: Box<Account<'info, ProtocolVaultState>>,

    /// System Program.
    pub system_program: Program<'info, System>,

    /// Rent sysvar (needed to compute the rent‑exempt minimum).
    pub rent: Sysvar<'info, Rent>,
}

impl<'info> LiquidateLoan<'info> {
    pub fn liquidate_loan(
        &mut self,
        sol_price: u64, // SOL price in USDC smallest unit (passed directly)
    ) -> Result<()> {
        //  Ensure the lender calling this is the one recorded in the loan request.
        self.check_lender()?;
        //  Ensure that liquidation is eligible (i.e. collateral's USD value is below 110% threshold).
        self.ensure_liquidate_eligible(sol_price)?;
        //  Calculate net distribution amounts from the collateral vault.
        //  We subtract the rent‑exempt minimum so that only the "excess" collateral is split.
        let (_lender_net, protocol_fee) = self.calculate_distribution()?;
        // Transfer the protocol fee from the collateral vault (PDA) to the protocol vault account.
        self.transfer_fee(protocol_fee)?;
        //At the end of the instruction, both the loan_request and collateral_vault accounts
        //    are closed and their remaining lamports (including rent deposits) are sent to the lender.

        self.remove_from_loan_registry()?;
        Ok(())
    }

    /// Verifies that the lender in the loan request matches the signer.
    fn check_lender(&self) -> Result<()> {
        require!(
            self.loan_request.lender == Some(self.lender.key()),
            ErrorCode::NotRightLender
        );
        Ok(())
    }

    /// Checks if the loan is eligible for liquidation.
    /// Liquidation is allowed if the USD value of the collateral is below 110% of the loan amount.
    fn ensure_liquidate_eligible(&self, sol_price: u64) -> Result<()> {
        let collateral_lamports = self.collateral_vault.to_account_info().lamports();
        // Convert lamports to SOL (1 SOL = 1e9 lamports) and then to USD.
        let collateral_usd_value = (collateral_lamports as f64 / 1e9) * (sol_price as f64);
        // Assume loan_request.loan_amount is expressed in USD.
        let liquidation_threshold = self.loan_request.loan_amount as f64 * 1.10;
        require!(
            collateral_usd_value < liquidation_threshold,
            ErrorCode::CannotLiquidateYet
        );
        Ok(())
    }

    
    fn calculate_distribution(&self) -> Result<(u64, u64)> {
        let collateral_info = self.collateral_vault.to_account_info();
        let total_collateral = collateral_info.lamports();
        let rent_exempt = self.rent.minimum_balance(collateral_info.data_len());
        require!(
            total_collateral > rent_exempt,
            ErrorCode::InsuffientCollateral
        );
        let collateral_net = total_collateral - rent_exempt;
        let protocol_fee = collateral_net
            .checked_mul(2)
            .and_then(|x| x.checked_div(110))
            .ok_or(ErrorCode::CalculationError)?;
        let lender_net = collateral_net
            .checked_sub(protocol_fee)
            .ok_or(ErrorCode::CalculationError)?;
        Ok((lender_net, protocol_fee))
    }

    /// Transfers the protocol fee from the collateral vault (a PDA) to the protocol vault account.
    /// We use Anchor's `CpiContext::new_with_signer` to supply the PDA's seeds.
    fn transfer_fee(&self, fee: u64) -> Result<()> {
        let cpi_accounts = Transfer {
            from: self.collateral_vault.to_account_info(),
            to: self.protocol_vault.to_account_info(),
        };

        // The collateral_vault PDA was derived with seeds:
        // [b"collateral_vault", loan_request.key().as_ref()] and its bump.
        let loan_request_key = self.loan_request.to_account_info().key;
        let seeds: &[&[u8]] = &[
            b"collateral_vault",
            loan_request_key.as_ref(),
            &[self.collateral_vault.bump],
        ];
        let signer_seeds = &[seeds];

        let cpi_ctx = CpiContext::new_with_signer(
            self.system_program.to_account_info(),
            cpi_accounts,
            signer_seeds,
        );
        transfer(cpi_ctx, fee)?;
        Ok(())
    }



    // remove the loan request from the loan registry
    fn remove_from_loan_registry(&mut self) -> Result<()> {
        let loan_request_key = self.loan_request.key();
        
        // Find the index of the loan request in the registry
        let position = self.loan_registry.loan_requests.iter()
            .position(|&pubkey| pubkey == loan_request_key)
            .ok_or(ErrorCode::NotFoundInRegistry)?;
        
        // Remove the loan request from the registry
        self.loan_registry.loan_requests.remove(position);
        
        // Decrement the total loans counter
        self.loan_registry.total_loans = self.loan_registry.total_loans
            .checked_sub(1)
            .ok_or(ErrorCode::CalculationError)?;
            
        Ok(())
    }
}
