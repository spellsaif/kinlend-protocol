use anchor_lang::prelude::*;

use crate::state::{LoanRequestState, CollateralVaultState};

#[derive(Accounts)]
#[instruction(loan_id:u64)]
pub struct CreateLoanRequest<'info> {

    //Borrower is the one who creates the loan request account so, he/she will be paying for account creation.
    #[account(mut)]
    borrower: Signer<'info>,

    //Creating LoanRequestState Account
    #[account(
        init, 
        space = 8 + LoanRequestState::INIT_SPACE,
        payer = borrower, 
        seeds = [b"loan_request", borrower.key().as_ref(), &loan_id.to_le_bytes()],  //PDA for LoanRequestState Account
        bump
    )]
    pub loan_request: Box<Account<'info, LoanRequestState>>,

    //creating Vault account
    #[account(
        init,
        payer = borrower,
        space = 8 + CollateralVaultState::INIT_SPACE,
        seeds = [b"collateral_vault", loan_request.key().as_ref()],
        bump
    )]
    pub collateral_vault: Box<Account<'info, CollateralVaultState>>,

    pub system_program: Program<'info, System>,

}