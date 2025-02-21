use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct CollateralVaultState {
    pub bump:u8,
}