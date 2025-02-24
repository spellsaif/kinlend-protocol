use anchor_lang::prelude::*;

use crate::state::{CollateralVaultState, LoanRegistryState, LoanRequestState};

use crate::errors::ErrorCode;

#[derive(Accounts)]
pub struct CancelLoanRequest<'info> {
    
    //Borrower will be signer since borrower only has permission to cancel his/her Loan Request
    pub borrower: Signer<'info>,

    //Loan Request
    #[account(
        mut,
        close = borrower,
        seeds = [b"loan_request", borrower.key().as_ref(), &loan_request.loan_id.to_le_bytes()],
        bump
    )]
    pub loan_request: Box<Account<'info, LoanRequestState>>,


    //Collateral Vault
    #[account(
        mut,
        close = borrower,
        seeds = [b"collateral_vault", loan_request.key().as_ref()],
        bump = collateral_vault.bump

    )]
    pub collateral_vault: Box<Account<'info, CollateralVaultState>>,

    #[account(
        mut,
        seeds = [b"loan_registry"],
        bump
    )]
    pub loan_registry: Box<Account<'info, LoanRegistryState>>,

    //program
    pub system_program: Program<'info, System>
}

impl<'info> CancelLoanRequest<'info> {

    //cancel loan request
    pub fn cancel_loan_request(&mut self) -> Result<()> {

        //check loan funded or not by checking whether lender is assgined to given loan request
        self.check_loan_funded()?;


        //todo: implementing logic to remove loan_request from loan registry

        Ok(())
    }

    //checking whether loan is funded or not
    pub fn check_loan_funded(&mut self) -> Result<()> {

        if self.loan_request.lender.is_some() {
            return Err(ErrorCode::AlreadyFunded.into());
        }

        Ok(())
    }




}