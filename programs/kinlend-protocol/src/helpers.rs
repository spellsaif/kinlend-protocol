use anchor_lang::prelude::*;

use crate::errors::ErrorCode;


pub fn check_deadline(deadline: i64) -> Result<()> {
    let clock = Clock::get()?;

    if clock.unix_timestamp > deadline {
        return Err(ErrorCode::RepaymentTimeExpired.into());
    }


    Ok(())
}


pub fn check_right_borrower(borrower: Pubkey, loan_request_borrower: Pubkey) -> Result<()> {

    if borrower != loan_request_borrower {
        return Err(ErrorCode::NotRightBorrower.into());
    }

    Ok(())
}


pub fn check_usdc_mint_address(config_usdc_mint:Pubkey, usdc_mint:Pubkey) -> Result<()> {

    if config_usdc_mint != usdc_mint {
        return Err(ErrorCode::IncorrectUsdcMintAddress.into());
    }

    Ok(())
}


pub fn check_balance(balance: u64, total_amount:u64) -> Result<()> {
        
        if balance < total_amounts {
            return Err(ErrorCode::InsufficientBalance.into());
        }
    Ok(())
}