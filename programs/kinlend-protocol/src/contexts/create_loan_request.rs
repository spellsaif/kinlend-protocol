use anchor_lang::prelude::*;

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

    #[account(
        mut,
        seeds = [b"loan_registry"],
        bump 
    )]
    pub loan_registry: Box<Account<'info, LoanRegistryState>>,

    #[account(
        mut,
        seeds = [b"loan_registry_page"],
        bump
    )]
    pub loan_registry_page: Box<Account<'info, LoanRegistryPageState>>,

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
        current_sol_price: u64

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

        Ok(())
    }
}