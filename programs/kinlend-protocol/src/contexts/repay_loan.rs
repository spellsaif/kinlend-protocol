use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::helpers::{check_deadline, check_right_borrower, check_usdc_mint_address};
use crate::state::{loan_request, CollateralVaultState, ConfigState, LoanRequestState, ProtocolVaultState};

use crate::errors::ErrorCode;

#[derive(Accounts)]
pub struct RepayLoan<'info> {

    //Since borrowing is making transaction
    #[account(mut)]
    pub borrower: Signer<'info>,

    //borrower's USDC ATA
    #[account(
        mut,
        constraint = borrower_usdc_account.mint == usdc_mint.key(),
        constraint = borrower_usdc_account.owner == borrower.key() 
    )]
    pub borrower_usdc_account: Box<Account<'info, TokenAccount>>,

    //lender's USDC ATA
    #[account(
        mut,
        constraint = lender_usdc_account.mint == usdc_mint.key()
    )]
    pub lender_usdc_account: Box<Account<'info, TokenAccount>>,


    //Loan Request account
    #[account(
        mut,
        seeds = [b"loan_request", loan_request.borrower.as_ref(), &loan_request.loan_id.to_le_bytes()],
        bump
    )]
    pub loan_request: Box<Account<'info, LoanRequestState>>,

    //Collateral Vault Account
    #[account(
        mut,
        seeds = [b"collateral_vault", loan_request.key().as_ref()],
        bump = collateral_vault.bump
    )]
    pub collateral_vault: Box<Account<'info, CollateralVaultState>>,

    #[account(
        mut,
        seeds = [b"protcol_vault"],
        bump = protocol_vault.bump
    )]
    pub protocol_vault: Box<Account<'info, ProtocolVaultState>>,

    //config account which stores mint address of usdc
    #[account(
        mut,
        seeds = [b"config"],
        bump,
    )]
    pub config: Box<Account<'info, ConfigState>>,

    //USDC_mint
    pub usdc_mint: Account<'info, Mint>,

    pub token_program: Program<'info, Token>,

    pub system_program: Program<'info, System>,
}


impl<'info> RepayLoan<'info> {

    pub fn repay_loan(&mut self) -> Result<()> {

        //can only be repaid by the borrower who has taken loan 
        let borrower = self.borrower.key();
        let loan_request_borrower = self.loan_request.borrower;
        check_right_borrower(borrower, loan_request_borrower)?;

        //checking deadline
        let deadline = self.loan_request.repayment_time.unwrap();

        check_deadline(deadline)?;

        //checking usdc_mint
        let config_usdc_mint = self.config.usdc_mint;
        let usdc_mint = self.usdc_mint.key();
        check_usdc_mint_address(config_usdc_mint, usdc_mint)?;



        Ok(())
    }

    
}