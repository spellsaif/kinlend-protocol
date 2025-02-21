use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct CollateralVaultState {
    bump:u8,
}