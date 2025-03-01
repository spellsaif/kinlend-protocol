use anchor_lang::prelude::*;

use crate::state::{CollateralVaultState, LoanRegistryState, LoanRequestState};

use crate::errors::ErrorCode;

#[derive(Accounts)]
// #[instruction(loan_id: u64)]
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
        self.remove_from_loan_registry()?;

        //todo: implementing logic to remove loan_request from loan registry

        Ok(())
    }

    //checking whether loan is funded or not
    fn check_loan_funded(&mut self) -> Result<()> {

        if self.loan_request.lender.is_some() {
            return Err(ErrorCode::AlreadyFunded.into());
        }

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