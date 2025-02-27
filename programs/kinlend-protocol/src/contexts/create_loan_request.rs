use anchor_lang::{prelude::*, solana_program::native_token::LAMPORTS_PER_SOL, system_program::{transfer, Transfer}};
use pyth_solana_receiver_sdk::price_update::{get_feed_id_from_hex, PriceUpdateV2};

use crate::{constants::{MAX_AGE, SOL_USD_FEED_ID}, errors::ErrorCode, state::{CollateralVaultState, LoanRegistryPageState, LoanRegistryState, LoanRequestState}};

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

    //Current page where we want to insert new loan request if exists
    #[account(mut)]
    pub loan_registry_page: Option<Box<Account<'info, LoanRegistryPageState>>>,

    //create new page if previous page is full
    #[account(
        init_if_needed,
        payer = borrower,
        space = 8 + LoanRegistryPageState::INIT_SPACE,
        seeds = [b"loan_registry_page", &(loan_registry.total_loans+1).to_le_bytes()[..]],
        bump
    )]
    pub new_registry_page: Option<Box<Account<'info, LoanRegistryPageState>>>,

    pub system_program: Program<'info, System>,

}



//Implementing CreateLoanRequest
impl<'info> CreateLoanRequest<'info> {
    pub fn create_loan_request(
        &mut self, 
        loan_id: u64, 
        loan_amount: u64, 
        collateral: u64,
        duration_days: u64,
        bumps: CreateLoanRequestBumps

    ) -> Result<()> {

        //retrieve current sol price from pyth oracle
        let current_sol_price = self.get_current_sol_price()?;

        //Calculate required collateral in SOL
        let required_sol = self.calculate_required_collateral(loan_amount, current_sol_price)?;

        
        //Ensure enough collateral provided
        self.verify_collateral(collateral, required_sol)?;


        //Initialize LoanRequestState Account
        self.initialize_loan_request(loan_id, loan_amount, collateral, duration_days)?;

        //Initialize CollateralVaultState Account
        self.initialize_collateral(bumps.collateral_vault)?;

        //Transfer collateral from borrower to collateral_vault PDA
        self.transfer_collateral_to_vault(collateral)?;


        //Update or add to LoanRegistryState
        self.update_loan_registry()?;

        //Incrementing total loans
        self.loan_registry.total_loans = self.loan_registry.total_loans.checked_add(1).ok_or(ErrorCode::CalculationError)?;


        Ok(())
    }
    
    fn get_current_sol_price(&self) -> Result<u64> {
        let feed_id = get_feed_id_from_hex(SOL_USD_FEED_ID)?;
        let clock = Clock::get()?;
        let price_data = self.price_update.get_price_no_older_than(&clock, MAX_AGE, &feed_id)?;
        Ok(price_data.price as u64)
    }
    
    fn calculate_required_collateral(&self, loan_amount: u64, current_sol_price: u64) -> Result<u64> {
        
        let required_collateral = loan_amount
            .checked_mul(150).ok_or(ErrorCode::CalculationError)?
            .checked_div(100).ok_or(ErrorCode::CalculationError)?
            .checked_div(current_sol_price).ok_or(ErrorCode::CalculationError)?;

        Ok(required_collateral)


    }
    
    fn verify_collateral(&self, collateral: u64, required_sol: u64) -> Result<()> {
        
        require!(collateral >= required_sol, ErrorCode::InsuffientCollateral);

        Ok(())
    }
    
    fn initialize_loan_request(&mut self, loan_id: u64, loan_amount: u64, collateral: u64, duration_days: u64) -> Result<()> {
        self.loan_request.set_inner(LoanRequestState{
            loan_id,
            loan_amount,
            collateral,
            duration_days,
            borrower: self.borrower.key(),
            lender: None,
            repayment_time: None
        });

        Ok(())
    }
    
    fn initialize_collateral(&mut self, bump:u8) -> Result<()> {
        self.collateral_vault.set_inner(CollateralVaultState{
            bump
        });

        Ok(())
    }
    
    fn transfer_collateral_to_vault(&mut self, collateral: u64) -> Result<()> {
        
        let cpi_accounts = Transfer{
            from: self.borrower.to_account_info(),
            to: self.collateral_vault.to_account_info()
        };

        let cpi_programs = self.system_program.to_account_info();

        let cpi_ctx = CpiContext::new(cpi_programs, cpi_accounts);

        transfer(cpi_ctx, collateral.checked_mul(LAMPORTS_PER_SOL).ok_or(ErrorCode::CalculationError)?)

    }
    
    fn update_loan_registry(&mut self) -> Result<()> {
        
        if let Some(page) = self.loan_registry_page.as_mut() {
            if page.loan_requests.len() < 10 {
                page.loan_requests.push(self.loan_request.key());
            } else {
                let new_page = self.new_registry_page.as_mut().ok_or(ErrorCode::PageIsFull)?;
                page.next_page = Some(new_page.key());
                new_page.loan_requests.push(self.loan_request.key());
            }
        } else {
            let new_page = self.new_registry_page.as_mut().ok_or(ErrorCode::PageIsFull)?;
            if self.loan_registry.first_page.is_none() {
                self.loan_registry.first_page = Some(new_page.key());
            }
            new_page.loan_requests.push(self.loan_request.key());
        }
        Ok(())
    }

}