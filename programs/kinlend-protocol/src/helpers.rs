use anchor_lang::prelude::*;

use crate::errors::ErrorCode;


pub fn check_deadline_is_expired(repayment_time: i64) -> Result<()> {
    
    // Compare current time with deadline.
    let clock = Clock::get()?;
    require!(
        clock.unix_timestamp > repayment_time,
        ErrorCode::LoanIsNotExpired
    );

    Ok(())
}

pub fn check_deadline_is_not_expired(repayment_time: i64)-> Result<()> {
    // Compare current time with deadline.
    let clock = Clock::get()?;
    require!(
        clock.unix_timestamp < repayment_time,
        ErrorCode::LoanExpired
    );

    Ok(())
}

pub fn calculate_repayment_time(duration_days: u64) -> Result<i64> {
     // Compute the deadline: now + (duration_days * 86400 seconds)
    let duration_seconds = duration_days
                                .checked_mul(86400)
                                .ok_or(ErrorCode::Overflow)? as i64;

    let clock = Clock::get()?;
    
    let now = clock.unix_timestamp;

    let deadline = duration_seconds
                        .checked_add(now)
                        .ok_or(ErrorCode::Overflow)?;
    

    Ok(deadline)


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
        
        if balance < total_amount {
            return Err(ErrorCode::InsufficientBalance.into());
        }
    Ok(())
}