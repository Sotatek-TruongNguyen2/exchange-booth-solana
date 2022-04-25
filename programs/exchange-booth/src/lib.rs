pub mod account;
pub mod context;
pub mod error;
pub mod helper;

use account::*;
use anchor_lang::prelude::*;
use anchor_spl::token::{transfer, Mint, TokenAccount};
use context::*;
use error::*;
use helper::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod exchange_booth {
    use super::*;

    pub fn initialize_exchange_booth(ctx: Context<InitializeExchange>, rate: u32) -> Result<()> {
        let vault_a: &Account<TokenAccount> = &ctx.accounts.vault_a;
        let vault_b: &Account<TokenAccount> = &mut ctx.accounts.vault_b;
        let token_a: &Account<Mint> = &ctx.accounts.token_a;
        let token_b: &Account<Mint> = &ctx.accounts.token_b;
        let admin: &Signer = &ctx.accounts.admin;

        let exchange_booth: &mut Account<ExchangeBooth> = &mut ctx.accounts.exchange_booth;

        exchange_booth.vault_a = vault_a.key();
        exchange_booth.vault_b = vault_b.key();
        exchange_booth.admin = admin.key();
        exchange_booth.token_a = token_a.key();
        exchange_booth.token_b = token_b.key();
        exchange_booth.vault_a_bump = *ctx.bumps.get("vault_a").unwrap();
        exchange_booth.vault_b_bump = *ctx.bumps.get("vault_b").unwrap();

        if rate == 0 {
            return Err(ProgramErrorCode::ZeroRateConfig.into());
        }

        exchange_booth.rate = rate;

        Ok(())
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        let admin_token_account: &Account<TokenAccount> = &ctx.accounts.admin_token_account;

        if admin_token_account.amount < amount {
            return Err(ProgramErrorCode::ExceedsCurrentBalance.into());
        }

        transfer(ctx.accounts.into_transfer_token_to_vault(), amount)?;

        Ok(())
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        let vault_account: &Account<TokenAccount> = &ctx.accounts.vault;
        let deposit_token: &Account<Mint> = &ctx.accounts.withdraw_token;
        let exchange_booth: &Account<ExchangeBooth> = &mut ctx.accounts.exchange_booth;

        if vault_account.amount < amount {
            return Err(ProgramErrorCode::ExceedsCurrentBalance.into());
        }

        let vault_bump_seed = match deposit_token.key() == exchange_booth.token_a {
            true => exchange_booth.vault_a_bump,
            _ => exchange_booth.vault_b_bump,
        };

        let seeds = &[
            match deposit_token.key() == exchange_booth.token_a {
                true => b"vault_a",
                _ => b"vault_b",
            },
            ctx.accounts.exchange_booth.to_account_info().key.as_ref(),
            ctx.accounts.withdraw_token.to_account_info().key.as_ref(),
            ctx.accounts.admin.to_account_info().key.as_ref(),
            &[vault_bump_seed],
        ];

        let signer = &[&seeds[..]];

        transfer(
            ctx.accounts
                .into_withdraw_token_from_vault()
                .with_signer(signer),
            amount,
        )?;

        Ok(())
    }

    pub fn exchange(ctx: Context<Exchange>, amount: u64) -> Result<()> {
        let token_a: &Account<Mint> = &ctx.accounts.token_a;
        let token_b: &Account<Mint> = &ctx.accounts.token_b;
        let vault_a: &Account<TokenAccount> = &ctx.accounts.vault_a;
        let vault_b: &Account<TokenAccount> = &mut ctx.accounts.vault_b;
        let exchange_booth: &Account<ExchangeBooth> = &ctx.accounts.exchange_booth;

        let exchanger_send_token_account: &Account<TokenAccount> =
            &ctx.accounts.exchanger_send_token_account;
        let exchanger_receive_token_account: &Account<TokenAccount> =
            &ctx.accounts.exchanger_receive_token_account;

        if exchanger_send_token_account.amount < amount {
            return Err(ProgramErrorCode::ExceedsCurrentBalance.into());
        }

        if exchanger_send_token_account.mint.key() == token_a.key() {
            if exchanger_receive_token_account.mint != token_b.key() {
                return Err(ProgramErrorCode::UnSupportedExchangeRoute.into());
            }

            let output_amount = calc_output_amount(amount, exchange_booth.rate, true);

            match output_amount {
                Some(amount) => {
                    if vault_b.amount < amount {
                        return Err(ProgramErrorCode::ExceedsCurrentFunds.into());
                    }

                    let seeds = &[
                        b"vault_b",
                        ctx.accounts.exchange_booth.to_account_info().key.as_ref(),
                        ctx.accounts.token_b.to_account_info().key.as_ref(),
                        ctx.accounts.exchange_booth.admin.as_ref(),
                        &[exchange_booth.vault_b_bump],
                    ];

                    let signer = &[&seeds[..]];

                    transfer(ctx.accounts.into_transfer_token_to_vault(true), amount)?;

                    transfer(
                        ctx.accounts
                            .into_transfer_token_from_vault_to_recipient(false)
                            .with_signer(signer),
                        amount,
                    )?;
                }
                None => return Err(ProgramErrorCode::InSufficientOutputAmount.into()),
            }
        } else if exchanger_send_token_account.mint.key() == token_b.key() {
            if exchanger_receive_token_account.mint != token_a.key() {
                return Err(ProgramErrorCode::UnSupportedExchangeRoute.into());
            }

            let output_amount = calc_output_amount(amount, exchange_booth.rate, true);

            match output_amount {
                Some(amount) => {
                    if vault_a.amount < amount {
                        return Err(ProgramErrorCode::ExceedsCurrentFunds.into());
                    }

                    let seeds = &[
                        b"vault_a",
                        ctx.accounts.exchange_booth.to_account_info().key.as_ref(),
                        ctx.accounts.token_a.to_account_info().key.as_ref(),
                        ctx.accounts.exchange_booth.admin.as_ref(),
                        &[exchange_booth.vault_a_bump],
                    ];

                    let signer = &[&seeds[..]];

                    transfer(ctx.accounts.into_transfer_token_to_vault(false), amount)?;

                    transfer(
                        ctx.accounts
                            .into_transfer_token_from_vault_to_recipient(true)
                            .with_signer(signer),
                        amount,
                    )?;
                }
                None => return Err(ProgramErrorCode::InSufficientOutputAmount.into()),
            }
        }

        Ok(())
    }
}
