use anchor_lang::prelude::*;

#[error_code]
pub enum ProgramErrorCode {
    #[msg("Rate can not be zero!")]
    ZeroRateConfig,
    #[msg("Exceeds current balance!")]
    ExceedsCurrentBalance,
    #[msg("You are not exchange booth admin!")]
    WrongExchangeBoothAdmin,
    #[msg("You are depositing to the wrong vault!")]
    WrongDepositVault,
    #[msg("This token not supported by exchange booth!")]
    NotSupportedToken,
    #[msg("This route not supported by smart contract!")]
    UnSupportedExchangeRoute,
    #[msg("Exceeds current funds of smart contract")]
    ExceedsCurrentFunds,
    #[msg("Output amount can not be caculated")]
    InSufficientOutputAmount,
}
