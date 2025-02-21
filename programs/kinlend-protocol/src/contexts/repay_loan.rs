use anchor_lang::prelude::*;
use anchor_spl::{token::Token, token_interface::{Mint, TokenAccount}};

use crate::state::{CollateralVaultState, LoanRequestState, ProtocolVaultState};

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

    //Lender's account
    #[account(mut)]
    pub lender: AccountInfo<'info>,

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

    //USDC_mint
    pub usdc_mint: Account<'info, Mint>,

    pub token_program: Program<'info, Token>,

    pub system_program: Program<'info, System>,
}