use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer};
use crate::helpers::check_deadline;
use crate::state::{
    CollateralVaultState, LoanRegistryState, LoanRequestState, ProtocolVaultState,
};
use crate::errors::ErrorCode;

#[derive(Accounts)]
pub struct ClaimCollateral<'info> {
    /// The lender claiming collateral. Must be the one recorded as the lender in the loan request.
    #[account(mut)]
    pub lender: Signer<'info>,

    /// Loan Request account (PDA) containing loan details.
    /// Derived using: [b"loan_request", loan_request.borrower.as_ref(), &loan_request.loan_id.to_le_bytes()]
    #[account(
        mut,
        seeds = [b"loan_request", loan_request.borrower.as_ref(), &loan_request.loan_id.to_le_bytes()],
        bump
    )]
    pub loan_request: Box<Account<'info, LoanRequestState>>,

    /// Collateral Vault (PDA) that holds the collateral (in SOL lamports).
    /// When closed, any remaining lamports are sent to the lender.
    #[account(
        mut,
        seeds = [b"collateral_vault", loan_request.key().as_ref()],
        bump = collateral_vault.bump
    )]
    pub collateral_vault: Account<'info, CollateralVaultState>,

    /// Protocol Vault account (PDA) that collects fees.
    #[account(
        mut,
        seeds = [b"protocol_vault"],
        bump
    )]
    pub protocol_vault: Box<Account<'info, ProtocolVaultState>>,

    /// Loan Registry account (tracking all loans).
    #[account(
        mut,
        seeds = [b"loan_registry"],
        bump,
    )]
    pub loan_registry: Box<Account<'info, LoanRegistryState>>,

    /// System Program for lamport transfers.
    pub system_program: Program<'info, System>,
}

impl<'info> ClaimCollateral<'info> {
    /// Main function to claim collateral.
    /// It ensures the loan is defaulted, obtains the collateral amount, and transfers:
    /// - 10% as a fee to the protocol vault,
    /// - 90% to the lender.
    pub fn claim_collateral(&mut self) -> Result<()> {
        self.ensure_loan_defaulted()?;
        let collateral_amount = self.get_collateral()?;
        self.transfer_collateral(collateral_amount)?;
        Ok(())
    }

    /// Verifies that:
    /// The loan is funded by the caller (lender).
    /// The loan is defaulted (i.e. current time is past the computed deadline).
    ///
    /// Deadline Calculation: 
    /// If your state stored the computed deadline, you would simply compare it.  
    /// Here we assume the state stores only the funding time and duration.
    pub fn ensure_loan_defaulted(&self) -> Result<()> {
        // Check that the loan request is funded by the calling lender.
        let funded_lender = self.loan_request.lender;
        require!(
            funded_lender == Some(self.lender.key()),
            ErrorCode::UnauthorizedLender
        );

        let repayment_time = self.loan_request.repayment_time.unwrap();
        let duration_days = self.loan_request.duration_days;

        check_deadline(repayment_time, duration_days)?;

        Ok(())
    }

    /// Retrieves the total lamport balance from the collateral vault.
    pub fn get_collateral(&self) -> Result<u64> {
        let vault_lamports = self.collateral_vault.to_account_info().lamports();
        require!(vault_lamports > 0, ErrorCode::NoCollateral);
        Ok(vault_lamports)
    }

    /// Transfers collateral from the collateral vault:
    /// - 90% goes to the lender,
    /// - 10% goes to the protocol vault.
    /// 
    /// The function calculates the fee and lender amount, then uses CPI to the system program
    /// with the collateral vault PDA signing the transfer.
    pub fn transfer_collateral(&self, total_amount: u64) -> Result<()> {
        // Calculate fee: 10% of the total collateral.
        let fee = total_amount
            .checked_div(10)
            .ok_or(ErrorCode::Overflow)?;
        // Calculate lender's share: remaining 90%.
        let lender_amount = total_amount
            .checked_sub(fee)
            .ok_or(ErrorCode::Overflow)?;

        // The collateral vault PDA's seeds.
        let loan_request_key = self.loan_request.key();
        let collateral_seeds: &[&[u8]] = &[
            b"collateral_vault",
            loan_request_key.as_ref(),
            &[self.collateral_vault.bump],
        ];
        let signer_seeds = &[collateral_seeds];

        // Transfer 90% to the lender.
        {
            let cpi_accounts = Transfer {
                from: self.collateral_vault.to_account_info(),
                to: self.lender.to_account_info(),
            };
            let cpi_ctx = CpiContext::new_with_signer(
                self.system_program.to_account_info(),
                cpi_accounts,
                signer_seeds,
            );
            transfer(cpi_ctx, lender_amount)?;
        }
        
        // Transfer 10% fee to the protocol vault.
        {
            let cpi_accounts = Transfer {
                from: self.collateral_vault.to_account_info(),
                to: self.protocol_vault.to_account_info(),
            };
            let cpi_ctx = CpiContext::new_with_signer(
                self.system_program.to_account_info(),
                cpi_accounts,
                signer_seeds,
            );
            transfer(cpi_ctx, fee)?;
        }
        Ok(())
    }
}
