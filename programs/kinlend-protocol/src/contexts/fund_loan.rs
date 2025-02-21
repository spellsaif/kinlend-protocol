use anchor_lang::prelude::*;
use anchor_spl::{token::Token, token_interface::{Mint, TokenAccount}};

use crate::state::LoanRequestState;

#[derive(Accounts)]
#[instruction(loan_id: u64)]
pub struct FundLoan<'info> {
    #[account(mut)]
    pub lender: Signer<'info>,

    #[account(
        mut,
        seeds = [b"loan_request", loan_request.borrower.as_ref(), &loan_id.to_le_bytes()],
        bump
    )]
    pub loan_request: Box<Account<'info, LoanRequestState>>,

    //The Borrower receives USDC
    #[account(mut)]
    pub borrower: AccountInfo<'info>,

    //Lender's associated token account
    #[account(
        mut,
        constraint = lender_usdc_account.mint == usdc_mint.key(),
        constraint = lender_usdc_account.owner == lender.key()
    )]
    pub lender_usdc_account: Box<Account<'info, TokenAccount>>,

    //Borrower's associated token account
    #[account(
        mut,
        constraint = borrower_usdc_account.mint == usdc_mint.key(),
    )]
    pub borrower_usdc_account: Box<Account<'info, TokenAccount>>,


    pub usdc_mint: Account<'info, Mint>,
    pub token_progran: Program<'info, Token>,
    pub system_program: Program<'info, System>,

}