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

        self.remove_loan_from_registry()?;

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


    fn remove_loan_from_registry(&mut self) -> Result<()> {
        let mut current_page = match self.loan_registry.first_page {
            Some(page) => page,
            None => return Ok(()), // No pages exist, nothing to remove
        };

        let mut prev_page: Option<AccountInfo<'info>> = None;

        loop {
            let page_account = Account::<'info, LoanRegistryPageState>::try_from(
                &self.loan_registry.to_account_info().owner.clone()
            )?;

            // Check if this page contains the loan_request
            if let Some(index) = page_account.loan_requests.iter().position(|&x| x == self.loan_request.key()) {
                page_account.loan_requests.remove(index); // Remove loan_request from the list

                // If the page is now empty, remove it from the linked list
                if page_account.loan_requests.is_empty() {
                    match prev_page {
                        Some(prev_page_account) => {
                            let mut prev_page_data = Account::<LoanRegistryPageState>::try_from(&prev_page_account)?;
                            prev_page_data.next_page = page_account.next_page; // Skip the empty page
                        }
                        None => {
                            self.loan_registry.first_page = page_account.next_page; // Update first_page if it's the first page
                        }
                    }
                }

                self.loan_registry.total_loans -= 1; // Decrease the total loan count

                return Ok(());
            }

            // Move to next page
            if let Some(next_page) = page_account.next_page {
                prev_page = Some(page_account.to_account_info());
                current_page = next_page;
            } else {
                break;
            }
        }

        Ok(())
    }
}