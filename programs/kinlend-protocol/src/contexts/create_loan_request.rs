use anchor_lang::{prelude::*, solana_program::native_token::LAMPORTS_PER_SOL, system_program::{transfer, Transfer}};
use pyth_solana_receiver_sdk::price_update::{get_feed_id_from_hex, PriceUpdateV2};

use crate::{constants::{MAX_AGE, SOL_USD_FEED_ID, USDC_USD_FEED_ID}, errors::ErrorCode, state::{CollateralVaultState, LoanRegistryPageState, LoanRegistryState, LoanRequestState}};

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

    //Pyth PriceUpdateV2 account
    pub price_update: Account<'info, PriceUpdateV2>,

    //Root registry Account
    #[account(
        mut,
        seeds = [b"loan_registry"],
        bump 
    )]
    pub loan_registry: Box<Account<'info, LoanRegistryState>>,


    pub system_program: Program<'info, System>,

}



//Implementing CreateLoanRequest
impl<'info> CreateLoanRequest<'info> {
    /// Creates a new loan request.
    ///
    /// Parameters:
    /// - `loan_id`: Unique loan identifier.
    /// - `loan_amount`: The desired loan amount in USDC smallest unit (e.g., 1 USDC = 1_000_000).
    /// - `collateral`: The collateral provided in lamports (e.g., 1 SOL = 1_000_000_000).  
    ///    This allows fractional SOL to be represented (e.g., 0.2 SOL = 200_000_000).
    /// - `duration_days`: The loan duration in days.
    ///
    /// **Collateral Requirement:**  
    /// The collateral must be at least 150% of the loan amount (in USDC terms).  
    /// We calculate the required collateral in lamports using the formula:
    ///
    /// required_collateral = (loan_amount * 150 * LAMPORTS_PER_SOL) / (100 * current_sol_price)
    ///
    /// where `current_sol_price` is retrieved from Pyth (and is assumed to be in USDC smallest unit per SOL).
    pub fn create_loan_request(
        &mut self, 
        loan_id: u64, 
        loan_amount: u64, 
        collateral: u64,       // collateral provided in lamports
        duration_days: u64,
        bumps: CreateLoanRequestBumps,
    ) -> Result<()> {
        // 1. Retrieve the current SOL price from Pyth.
        let current_sol_price = self.get_current_sol_price()?;
        
        // 2. Calculate the required collateral (in lamports) based on the loan amount.
        let required_collateral = self.calculate_required_collateral(loan_amount, current_sol_price)?;
        
        // 3. Verify that the provided collateral is sufficient.
        self.verify_collateral(collateral, required_collateral)?;
        
        // 4. Initialize the LoanRequest state.
        self.initialize_loan_request(loan_id, loan_amount, collateral, duration_days)?;
        
        // 5. Initialize the CollateralVault state.
        self.initialize_collateral_vault(bumps.collateral_vault)?;
        
        // 6. Transfer the provided collateral from the borrower’s wallet into the collateral vault.
        self.transfer_collateral_to_vault(collateral)?;
        
        // 7. Update the loan registry by adding the new loan request's key.
        self.update_loan_registry()?;
        
        // 8. Increment the total number of loans in the registry.
        self.loan_registry.total_loans = self.loan_registry.total_loans
            .checked_add(1)
            .ok_or(ErrorCode::CalculationError)?;
        
        Ok(())
    }
    
    /// Retrieves the current SOL price from the Pyth price feed.
    /// Assumes the price is returned in USDC smallest unit per SOL.
    fn get_current_sol_price(&self) -> Result<u64> {
        let clock = Clock::get()?;
        let feed_id = get_feed_id_from_hex(SOL_USD_FEED_ID)?;
        let price_data = self.price_update.get_price_no_older_than(&clock, MAX_AGE, &feed_id)?;
        Ok(price_data.price as u64)
    }
    
    /// Calculates the required collateral (in lamports) using the formula:
    /// required_collateral = (loan_amount * 150 * LAMPORTS_PER_SOL) / (100 * current_sol_price)
    /// - loan_amount: in USDC smallest unit (e.g., 1 USDC = 1_000_000)
    /// - current_sol_price: in USDC smallest unit per SOL (e.g., 20 USDC per SOL = 20_000_000)
    fn calculate_required_collateral(&self, loan_amount: u64, current_sol_price: u64) -> Result<u64> {
        let numerator = loan_amount
            .checked_mul(150).ok_or(ErrorCode::CalculationError)?
            .checked_mul(LAMPORTS_PER_SOL).ok_or(ErrorCode::CalculationError)?;
        let denominator = 100_u64
            .checked_mul(current_sol_price).ok_or(ErrorCode::CalculationError)?;
        numerator.checked_div(denominator).ok_or(ErrorCode::CalculationError.into())
    }
    
    /// Verifies that the provided collateral (in lamports) is at least the required amount.
    fn verify_collateral(&self, provided: u64, required: u64) -> Result<()> {
        require!(provided >= required, ErrorCode::InsuffientCollateral);
        Ok(())
    }
    
    /// Initializes the LoanRequest state.
    fn initialize_loan_request(&mut self, loan_id: u64, loan_amount: u64, collateral: u64, duration_days: u64) -> Result<()> {
        self.loan_request.set_inner(LoanRequestState {
            loan_id,
            loan_amount,
            collateral, // stored in lamports
            duration_days,
            borrower: self.borrower.key(),
            lender: None,
            repayment_time: None,
        });
        Ok(())
    }
    
    /// Initializes the CollateralVault state.
    fn initialize_collateral_vault(&mut self, bump:u8) -> Result<()> {
        self.collateral_vault.set_inner(CollateralVaultState {
            bump
        });
        Ok(())
    }
    
    /// Transfers the collateral (in lamports) from the borrower’s wallet to the collateral vault.
    fn transfer_collateral_to_vault(&self, collateral: u64) -> Result<()> {
        let cpi_accounts = Transfer {
            from: self.borrower.to_account_info(),
            to: self.collateral_vault.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(self.system_program.to_account_info(), cpi_accounts);
        transfer(cpi_ctx, collateral)
    }
    
    /// Updates the loan registry by adding the new loan request's key.
    fn update_loan_registry(&mut self) -> Result<()> {
        self.loan_registry.loan_requests.push(self.loan_request.key());
        Ok(())
    }
}