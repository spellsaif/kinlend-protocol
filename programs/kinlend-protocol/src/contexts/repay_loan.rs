use anchor_lang::prelude::*;
use anchor_spl::token::{transfer, Mint, Token, TokenAccount, Transfer};

use crate::helpers::{check_balance, check_deadline, check_right_borrower, check_usdc_mint_address};
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
        close = borrower,
        seeds = [b"collateral_vault", loan_request.key().as_ref()],
        bump = collateral_vault.bump
    )]
    pub collateral_vault: Box<Account<'info, CollateralVaultState>>,

    //protocol vault USDC
    #[account(
        init_if_needed,
        payer = borrower,
        token::mint = usdc_mint,
        token::authority = protocol_vault_authority,
        seeds = [b"protocol_vault_usdc"],
        bump
    )]
    pub protocol_vault_usdc: Box<Account<'info, TokenAccount>>,


    //authority for protocol_vault_usdc
    pub protocol_vault_authority: AccountInfo<'info>,

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

    pub rent: Sysvar<'info, Rent>
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

        //calculating repayment amount
        let (lender_amount, fee, total_amount) = self.calculate_repayment_amounts()?;

        //check borrower balance
        let borrower_usdc_balance = self.borrower_usdc_account.amount;
        check_balance(borrower_usdc_balance, total_amount)?;

        // Transfer USDC tokens:
        //    - 104% of the loan amount goes to the lender.
        //    - 1% of the loan amount (as fee) goes to the protocol vault.
        self.transfer_tokens(lender_amount, fee)?;
        
        // When the instruction completes, the collateral_vault account is automatically closed,
        // and its entire lamport balance is transferred to the borrower because of `close = borrower`.


        Ok(())
    }


    /// Calculate repayment amounts:
    /// - lender_amount: 104% of the loan amount (USDC).
    /// - fee: 1% of the loan amount (USDC).
    /// - total_amount: Sum of the two (used for balance checks).
    pub fn calculate_repayment_amounts(&mut self) -> Result<(u64,u64, u64)> {

        let loan_amount = self.loan_request.loan_amount;
        let lender_amount = loan_amount
                        .checked_mul(104)
                        .and_then(|x| x.checked_div(100))
                        .ok_or(ErrorCode::CalculationError)?;
        
        let fee = loan_amount
                    .checked_mul(1)
                    .and_then(|x| x.checked_div(100))
                    .ok_or(ErrorCode::CalculationError)?;
        
        let total_amount = lender_amount.checked_add(fee).ok_or(ErrorCode::CalculationError)?;

        Ok((lender_amount,fee,total_amount))
    }


    pub fn transfer_tokens(&mut self, lender_amount: u64, fee:u64) -> Result<()> {

        //all needed accounts
        let cpi_accounts = Transfer{
            from: self.borrower_usdc_account.to_account_info(),
            to: self.lender_usdc_account.to_account_info(),
            authority: self.borrower.to_account_info()
        };

        //token program for transferring usdc to token
        let cpi_program = self.token_program.to_account_info();

        //Creating CPI Context for Cross Program Invocation
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        transfer(cpi_ctx, lender_amount)?;

        Ok(())


        //doing same for transferring usdc to protocol vault usdc account
        let cpi_accounts = Transfer {
            from: self.borrower_usdc_account.to_account_info(),
            to: self.protocol_vault_usdc.to_account_info(),
            authority: self.borrower.to_account_info(),
        };

        let cpi_program = self.token_program.to_account_info();

        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        transfer(cpi_ctx, fee)?;


        Ok(())
    }
    
}