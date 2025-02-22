use anchor_lang::{prelude::*, solana_program::native_token::LAMPORTS_PER_SOL, system_program::{transfer, Transfer}};

use crate::{errors::ErrorCode, state::{CollateralVaultState, LoanRegistryPageState, LoanRegistryState, LoanRequestState}};

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
        seeds = [b"loan_registry_page", &loan_registry.total_loans.to_le_bytes()[..]],
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
        current_sol_price: u64,
        bumps: CreateLoanRequestBumps

    ) -> Result<()> {

        //caculating required collateral in SOL
        let required_sol = loan_amount
                                .checked_mul(150).ok_or(ErrorCode::CalculationError)?
                                .checked_div(100).ok_or(ErrorCode::CalculationError)?
                                .checked_div(current_sol_price).ok_or(ErrorCode::CalculationError)?;

        //Checking whether enough collateral is provided against borrowing amount.
        require!(collateral >= required_sol, ErrorCode::InsuffientCollateral);
        let borrower = &self.borrower;

        //Creating LoanRequestState Account which holds all the information regarding particular loan request
        self.loan_request.set_inner(LoanRequestState {
            loan_id,
            loan_amount,
            duration_days,
            collateral,
            borrower: borrower.key(),
            lender: None,
            repayment_time: None
        });

        //Creating CollateralVaultState
        self.collateral_vault.set_inner(CollateralVaultState{
            bump: bumps.collateral_vault
        });


        //transferring collateral from borrower's wallet to collateral vault

        //Accounts needed
        let cpi_accounts = Transfer{
            from: self.borrower.to_account_info(),
            to: self.collateral_vault.to_account_info()
        };

        //System Program for making cpi calls
        let cpi_program = self.system_program.to_account_info();

        //cpi context
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        //now initiating transfer
        transfer(cpi_ctx, collateral * LAMPORTS_PER_SOL)?;


        //adding to LoanRegistryPage
        if let Some(page) = self.loan_registry_page.as_mut() {
            //Add to current page if it's not full
            if page.loan_requests.len() < 10 {
                page.loan_requests.push(self.loan_request.key());
            } else {
                //if page is full, new registry page
                let new_page = self.new_registry_page.as_mut().ok_or(ErrorCode::PageIsFull)?;

                //Linking new page from full one
                page.next_page = Some(new_page.key());

                //adding loan request to new page
                new_page.loan_requests.push(self.loan_request.key());

            }
        } else {
            
            //if no loan registry page exists, use the new page
            let new_page =  self.loan_registry_page.as_mut().ok_or(ErrorCode::PageIsFull)?;

            //if not first page exists, make this first page in loan registry
            if self.loan_registry.first_page.is_none() {
                self.loan_registry.first_page = Some(new_page.key());
            }

            //add loan request to new page
            new_page.loan_requests.push(self.loan_request.key());
        }


        self.loan_registry.total_loans += 1;


        Ok(())
    }
}