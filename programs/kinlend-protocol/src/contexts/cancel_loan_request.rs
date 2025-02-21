use anchor_lang::prelude::*;

use crate::state::{CollateralVaultState, LoanRegistryState, LoanRequestState};

#[derive(Accounts)]
pub struct CancelLoanRequest<'info> {
    
    //Borrower will be signer since borrower only has permission to cancel his/her Loan Request
    pub borrower: Signer<'info>,

    //Loan Request
    #[account(
        mut,
        seeds = [b"loan_request", borrower.key().as_ref(), &loan_request.loan_id.to_le_bytes()],
        bump
    )]
    pub loan_request: Box<Account<'info, LoanRequestState>>,


    //Collateral Vault
    #[account(
        mut,
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