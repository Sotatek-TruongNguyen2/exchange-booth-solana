use anchor_lang::prelude::*;

#[account]
pub struct ExchangeBooth {
    pub vault_a_bump: u8,
    pub vault_b_bump: u8,
    pub token_a: Pubkey,
    pub token_b: Pubkey,
    pub vault_a: Pubkey,
    pub vault_b: Pubkey,
    pub admin: Pubkey,
    pub rate: u32,
}

// 2. Add some useful constants for sizing propeties.
const BUMP_LENGTH: usize = 8;
const PUBLIC_KEY_LENGTH: usize = 32;
const DISCRIMINATOR_LENGTH: usize = 8;
const RATE_LENGTH: usize = 32;

impl ExchangeBooth {
    pub const LEN: usize =
        DISCRIMINATOR_LENGTH + RATE_LENGTH + PUBLIC_KEY_LENGTH * 5 + BUMP_LENGTH * 2;
}
