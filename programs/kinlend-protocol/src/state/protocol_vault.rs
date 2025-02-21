use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct ProtocolVaultState {
    pub bump: u8,
}