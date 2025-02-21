use anchor_lang::prelude::*;

use crate::state::LoanRequestState;

#[derive(Accounts)]
pub struct FundLoan<'info> {
    #[account(mut)]
    pub lender: Signer<'info>,

    #[account(mut)]
    pub loan_requets: Box<Account<'info, LoanRequestState>>,

    pub system_program: Program<'info, System>,
}