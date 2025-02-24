use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer};
use pyth_solana_receiver_sdk::price_update::{get_feed_id_from_hex, PriceUpdateV2};

use crate::state::{ CollateralVaultState, LoanRegistryState, LoanRequestState, ProtocolVaultState};

use crate::errors::ErrorCode;

#[derive(Accounts)]
pub struct LiquidateLoan<'info> {


    #[account(mut)]
    pub lender: Signer<'info>,

    #[account(
        mut,
        close = lender
    )]
    pub loan_request: Box<Account<'info, LoanRequestState>>,

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

    #[account(
        mut,
        seeds = [b"protocol_vault"],
        bump = protocol_vault.bump
    )]
    pub protocol_vault: Box<Account<'info, ProtocolVaultState>>,

    pub price_update: Account<'info, PriceUpdateV2>,

    //program
    pub system_program: Program<'info, System>,

    
}


//implementation
impl<'info> LiquidateLoan<'info> {
    pub fn liquidate_loan(&mut self) -> Result<()> {
        //checking for liquidation eligibility
        self.ensure_liquidate_eligible()?;

        // get protocol fee and collateral for lender
        let (lender_amount, protocol_fee) = self.calculate_distribution()?;

        // Transfer funds
        self.transfer_funds(self.lender.to_account_info(), lender_amount)?;
        self.transfer_funds(self.protocol_vault.to_account_info(), protocol_fee)?;

        Ok(())
        
    }

    //Checking if it is eligible for liquidation
    fn ensure_liquidate_eligible(&self) -> Result<()> {
        let sol_price = self.get_current_sol_price()?;
        let collateral_value = self.collateral_vault.to_account_info().lamports();
        let collateral_usd_value = collateral_value * sol_price;
        let liquidation_threshold = self.loan_request.loan_amount * 110 / 100;

        require!(
            collateral_usd_value < liquidation_threshold,
            ErrorCode::CannotLiquidateYet
        );

        Ok(())
    }

    ///Calculating distribution of sol for lender and protocol
    fn calculate_distribution(&self) -> Result<(u64, u64)> {
        let collateral_value = self.collateral_vault.to_account_info().lamports();
        let lender_amount = collateral_value * 108 / 110;
        let protocol_fee = collateral_value - lender_amount;

        Ok((lender_amount, protocol_fee))
    }


     /// Fetch current SOL price from Pyth
     fn get_current_sol_price(&self) -> Result<u64> {
        let maximum_age: u64 = 30;
        const SOL_USD_HEX: &str = "0xef0d8b6fda2ceba41da15d4095d1da392a0d2f8ed0c6c7bc0f4cfac8c280b56d";
        
        let feed_id = get_feed_id_from_hex(SOL_USD_HEX)?;
        let price_data = self.price_update.get_price_no_older_than(&Clock::get()?, maximum_age, &feed_id)?;
        
        Ok(price_data.price as u64)
    }

    fn transfer_funds(&self, recipient: AccountInfo<'info>, amount: u64) -> Result<()> {
        let cpi_accounts = Transfer {
            to: recipient,
            from: self.collateral_vault.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(self.system_program.to_account_info(), cpi_accounts);

        transfer(cpi_ctx, amount) 
    }
}