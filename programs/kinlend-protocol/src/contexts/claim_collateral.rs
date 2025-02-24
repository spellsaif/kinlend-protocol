use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer};

use crate::state::{ CollateralVaultState, LoanRegistryState, LoanRequestState, ProtocolVaultState};

use crate::errors::ErrorCode;

#[derive(Accounts)]
pub struct ClaimCollateral<'info> {

    //Lender will be the one who will be able to claim collateral if borrower fails to repay
    //Lender is signer here
    #[account(mut)]
    pub lender: Signer<'info>,

    //Loan Request Account
    #[account(
        mut,
        seeds = [b"loan_request", loan_request.borrower.as_ref(), &loan_request.loan_id.to_le_bytes()],
        bump
    )]
    pub loan_request: Box<Account<'info, LoanRequestState>>,

    //Collateral Vault
    #[account(
        mut,
        seeds = [b"collateral_vault", loan_request.key().as_ref()],
        bump = collateral_vault.bump
    )]
    pub collateral_vault: Account<'info, CollateralVaultState>,

    #[account(
        mut,
        seeds = [b"protocol_vault"],
        bump
    )]
    pub protocol_vault: Box<Account<'info, ProtocolVaultState>>,

    #[account(
        mut,
        seeds = [b"loan_registry"],
        bump,
    )]
    pub loan_registry: Box<Account<'info, LoanRegistryState>>,

    //program
    pub system_program: Program<'info, System>
}


impl<'info> ClaimCollateral<'info> {
    pub fn claim_protocol(&mut self) -> Result<()> {

        self.ensure_loan_defaulted()?;
        let collateral_amount = self.get_collateral()?;
        self.transfer_collateral(collateral_amount)?;
        Ok(())
    }

    pub fn ensure_loan_defaulted(&mut self) -> Result<()> {

        //Ensure loan is funded
        let funded_lender = self.loan_request.lender;

        if funded_lender != Some(self.lender.key()) {
            return Err(ErrorCode::UnauthorizedLender.into());
        }

        let repayment_time = self.loan_request.repayment_time.ok_or(ErrorCode::NotFunded)?;

        //Calculate deadline: repayment_time + (duration_days * 86400 seconds)
        let duration_seconds = self.loan_request.duration_days
                    .checked_mul(86400)
                    .ok_or(ErrorCode::Overflow)? as i64;
        
        let deadline = repayment_time
            .checked_add(duration_seconds)
            .ok_or(ErrorCode::Overflow)?;

        let clock = Clock::get()?;

        if clock.unix_timestamp <= deadline {
            return Err(ErrorCode::LoanIsNotExpired.into());
        }
        
        Ok(())
    }

    pub fn get_collateral(&mut self) -> Result<(u64)> {

        let vault_lamports = self.collateral_vault.to_account_info().lamports();

        if vault_lamports == 0 {
            return Err(ErrorCode::NoCollateral.into());
        } 

        Ok(vault_lamports)
    }

    pub fn transfer_collateral(&mut self, total_amount: u64) -> Result<()> {

        //Calculate fee as 10% of total amount
        let fee = total_amount
                .checked_div(10)
                .ok_or(ErrorCode::Overflow)?;

        let lender_amount = total_amount
                .checked_sub(fee)
                .ok_or(ErrorCode::Overflow)?;

        
        let loan_request_key = self.loan_request.key();
        
        //Derive the seeds for the collateral vault PDA to sign the CPI.
        let collateral_vault_seeds:&[&[u8]] = &[
            b"collateral_vault",
            loan_request_key.as_ref(),
            &[self.collateral_vault.bump]
        ];

        let seed_signer = &[collateral_vault_seeds];



        //Transfer 90% to the lender
        let cpi_accounts = Transfer {
            from: self.collateral_vault.to_account_info(),
            to: self.lender.to_account_info()
        };

        let cpi_program = self.system_program.to_account_info();

        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, seed_signer);

        transfer(cpi_ctx, lender_amount)?;
        
        
        //Transfer 10% to the protocol vault as fee
        let cpi_accounts = Transfer{
            from: self.collateral_vault.to_account_info(),
            to: self.protocol_vault.to_account_info()
        };

        let cpi_program = self.system_program.to_account_info();

        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, seed_signer);

        transfer(cpi_ctx, fee)?;

        Ok(())
    }
}