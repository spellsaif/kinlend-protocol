use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct ProtocolVaultState {
    bump: u8,
}