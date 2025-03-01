use anchor_lang::{prelude::*, solana_program::native_token::LAMPORTS_PER_SOL, system_program::{transfer, Transfer}};

use crate::{errors::ErrorCode, state::{CollateralVaultState, LoanRegistryState, LoanRequestState}};

#[derive(Accounts)]
#[instruction(loan_id:u64)]
pub struct CreateLoanRequest<'info> {
    // Borrower is the one who creates the loan request account so, he/she will be paying for account creation.
    #[account(mut)]
    borrower: Signer<'info>,

    // Creating LoanRequestState Account
    #[account(
        init, 
        space = 8 + LoanRequestState::INIT_SPACE,
        payer = borrower, 
        seeds = [b"loan_request", borrower.key().as_ref(), &loan_id.to_le_bytes()],  // PDA for LoanRequestState Account
        bump
    )]
    pub loan_request: Box<Account<'info, LoanRequestState>>,

    // Creating Vault account
    #[account(
        init,
        payer = borrower,
        space = 8 + CollateralVaultState::INIT_SPACE,
        seeds = [b"collateral_vault", loan_request.key().as_ref()],
        bump
    )]
    pub collateral_vault: Box<Account<'info, CollateralVaultState>>,

    // Root registry Account
    #[account(
        mut,
        seeds = [b"loan_registry"],
        bump 
    )]
    pub loan_registry: Box<Account<'info, LoanRegistryState>>,

    pub system_program: Program<'info, System>,
}

// Implementing CreateLoanRequest
impl<'info> CreateLoanRequest<'info> {
        pub fn create_loan_request(
        &mut self, 
        loan_id: u64, 
        loan_amount: u64, 
        collateral: u64,       // collateral provided in lamports
        duration_days: u64,
        sol_price: u64,        // SOL price in USDC smallest unit (passed directly)
        bumps: CreateLoanRequestBumps,
    ) -> Result<()> {
        // 1. Calculate the required collateral (in lamports) based on the loan amount.
        let required_collateral = self.calculate_required_collateral(loan_amount, sol_price)?;
        
        // 2. Verify that the provided collateral is sufficient.
        self.verify_collateral(collateral, required_collateral)?;
        
        // 3. Initialize the LoanRequest state.
        self.initialize_loan_request(loan_id, loan_amount, collateral, duration_days)?;
        
        // 4. Initialize the CollateralVault state.
        self.initialize_collateral_vault(bumps.collateral_vault)?;
        
        // 5. Transfer the provided collateral from the borrower's wallet into the collateral vault.
        self.transfer_collateral_to_vault(collateral)?;
        
        // 6. Update the loan registry by adding the new loan request's key.
        self.update_loan_registry()?;
        
        // 7. Increment the total number of loans in the registry.
        self.loan_registry.total_loans = self.loan_registry.total_loans
            .checked_add(1)
            .ok_or(ErrorCode::CalculationError)?;
        
        Ok(())
    }
    
    /// Calculates the required collateral (in lamports) using the formula:
    /// required_collateral = (loan_amount * 150 * LAMPORTS_PER_SOL) / (100 * sol_price)
    /// - loan_amount: in USDC smallest unit (e.g., 1 USDC = 1_000_000)
    /// - sol_price: in USDC smallest unit per SOL (e.g., 20 USDC per SOL = 20_000_000)
    fn calculate_required_collateral(&self, loan_amount: u64, sol_price: u64) -> Result<u64> {
        let numerator = loan_amount
            .checked_mul(150).ok_or(ErrorCode::CalculationError)?
            .checked_mul(LAMPORTS_PER_SOL).ok_or(ErrorCode::CalculationError)?;
        let denominator = 100_u64
            .checked_mul(sol_price).ok_or(ErrorCode::CalculationError)?;
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
    
    /// Transfers the collateral (in lamports) from the borrower's wallet to the collateral vault.
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
