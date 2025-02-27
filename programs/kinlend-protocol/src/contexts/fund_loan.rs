use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{transfer, Mint, Token, TokenAccount, Transfer};

use crate::helpers::{calculate_repayment_time, check_usdc_mint_address};
use crate::state::{ConfigState, LoanRequestState};
use crate::errors::ErrorCode;

#[derive(Accounts)]
#[instruction(loan_id: u64)]
pub struct FundLoan<'info> {
    /// The lender funding the loan. Must sign the transaction.
    #[account(mut)]
    pub lender: Signer<'info>,

    /// Configuration account storing protocol settings.
    /// It is a PDA seeded with "config".
    #[account(
        mut,
        seeds = [b"config"],
        bump
    )]
    pub config: Box<Account<'info, ConfigState>>,

    /// Loan Request account representing the loan.
    /// It is a PDA derived using the borrower's key and the loan_id.
    #[account(
        mut,
        seeds = [b"loan_request", loan_request.borrower.as_ref(), &loan_id.to_le_bytes()],
        bump
    )]
    pub loan_request: Box<Account<'info, LoanRequestState>>,

    /// Borrower’s main account (system account). 
    /// This account will receive USDC tokens through its associated token account.
    #[account(mut)]
    pub borrower: SystemAccount<'info>,

    /// Lender's USDC associated token account.
    #[account(
        mut,
        constraint = lender_usdc_account.mint == usdc_mint.key(),
        constraint = lender_usdc_account.owner == lender.key()
    )]
    pub lender_usdc_account: Box<Account<'info, TokenAccount>>,

    /// Borrower’s USDC associated token account.
    /// This account is initialized if needed and is derived from the borrower's address.
    #[account(
        init_if_needed,
        payer = lender,
        associated_token::mint = usdc_mint,
        associated_token::authority = borrower
    )]
    pub borrower_usdc_account: Box<Account<'info, TokenAccount>>,

    /// The USDC Mint account.
    pub usdc_mint: Account<'info, Mint>,

    /// Program for token operations.
    pub token_program: Program<'info, Token>,
    /// Program for associated token account operations.
    pub associated_token_program: Program<'info, AssociatedToken>,
    /// System program.
    pub system_program: Program<'info, System>,
}

impl<'info> FundLoan<'info> {
    pub fn fund_loan(&mut self) -> Result<()> {
        // Ensure the loan hasn't already been funded.
        self.verify_not_funded()?;

        //Verify the provided USDC mint matches the one in the configuration.
        self.verify_usdc_mint()?;

        //Record the lender's key in the loan request.
        self.update_loan_request_with_lender()?;

        //Set the repayment_time to the current unix timestamp.
        self.update_repayment_time()?;

        //Transfer USDC from the lender's token account to the borrower's token account.
        self.transfer_usdc_funds(self.loan_request.loan_amount)?;

        Ok(())
    }

    /// Verifies that the loan request has not yet been funded.
    fn verify_not_funded(&self) -> Result<()> {
        // loan_request.lender must be None before funding.
        require!(self.loan_request.lender.is_none(), ErrorCode::AlreadyFunded);
        Ok(())
    }

    /// Verifies that the USDC mint provided in the instruction matches the configuration.
    fn verify_usdc_mint(&self) -> Result<()> {
        let config_usdc_mint = self.config.usdc_mint;
        let provided_usdc_mint = self.usdc_mint.key();
        check_usdc_mint_address(config_usdc_mint, provided_usdc_mint)
    }

    /// Updates the loan request state by storing the lender's public key.
    fn update_loan_request_with_lender(&mut self) -> Result<()> {
        self.loan_request.lender = Some(self.lender.key());
        Ok(())
    }

    /// Updates the repayment_time field with the current Unix timestamp.
    fn update_repayment_time(&mut self) -> Result<()> {
        let repayment_time = calculate_repayment_time(self.loan_request.duration_days)?;
        self.loan_request.repayment_time = Some(repayment_time);
        Ok(())
    }

    /// Transfers USDC from the lender's associated token account to the borrower's associated token account.
    /// The amount transferred equals the loan amount specified in the loan request.
    fn transfer_usdc_funds(&self, amount: u64) -> Result<()> {
        let cpi_accounts = Transfer {
            from: self.lender_usdc_account.to_account_info(),
            to: self.borrower_usdc_account.to_account_info(),
            authority: self.lender.to_account_info(),
        };

        let cpi_program = self.token_program.to_account_info();

        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        transfer(cpi_ctx, amount)
    }
}
