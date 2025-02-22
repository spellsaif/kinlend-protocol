use anchor_lang::prelude::*;
use anchor_spl::token::{transfer, Mint, Token, TokenAccount, Transfer};

use crate::state::LoanRequestState;
use crate::errors::ErrorCode;

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
    pub borrower: SystemAccount<'info>,

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

impl<'info> FundLoan<'info> {
    pub fn fund_loan(&mut self) -> Result<()> {
    
        //Loan Request Account
        let loan_request = &mut self.loan_request;
        
        require!(loan_request.lender.is_none(), ErrorCode::AlreadyFunded);

        //storing the lender in the loan request
        loan_request.lender = Some(self.lender.key());

        // Transfer USDC from lender to borrower
        let cpi_accounts = Transfer{
            from: self.lender_usdc_account.to_account_info(),
            to: self.borrower_usdc_account.to_account_info(),
            authority: self.lender.to_account_info()
        };

        let cpi_program = self.token_progran.to_account_info();

        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        transfer(cpi_ctx, loan_request.loan_amount)?;

        Ok(())
    }
}