use crate::error::*;
use crate::account::*;
use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount, Transfer};

#[derive(Accounts)]
pub struct InitializeExchange<'info> {
    #[account(init, payer = admin, space = ExchangeBooth::LEN)]
    pub exchange_booth: Account<'info, ExchangeBooth>,

    #[account(mut)]
    pub admin: Signer<'info>,

    pub token_a: Account<'info, Mint>,
    pub token_b: Account<'info, Mint>,

    #[account(
        init,
        payer = admin,
        seeds = [
            b"vault_a",
            exchange_booth.key().as_ref(),
            token_a.key().as_ref(),
            admin.key().as_ref()
        ],
        bump,
        token::mint = token_a,
        token::authority = vault_a
    )]
    pub vault_a: Box<Account<'info, TokenAccount>>,
    #[account(
        init,
        payer = admin,
        seeds = [
            b"vault_b",
            exchange_booth.key().as_ref(),
            token_b.key().as_ref(),
            admin.key().as_ref()
        ],
        bump, 
        token::mint = token_b,
        token::authority = vault_b,
    )]
    pub vault_b: Box<Account<'info, TokenAccount>>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(has_one = admin @ProgramErrorCode::WrongExchangeBoothAdmin)]
    pub exchange_booth: Account<'info, ExchangeBooth>,

    #[account( 
        mut, 
        seeds = [
            match deposit_token.key() == exchange_booth.token_a {
                true => b"vault_a",
                _ => b"vault_b"
            },  
            exchange_booth.key().as_ref(),
            deposit_token.key().as_ref(),
            admin.key().as_ref()
        ],
        bump = match deposit_token.key() == exchange_booth.token_a {
            true => exchange_booth.vault_a_bump,
            _ => exchange_booth.vault_b_bump
        },  
        constraint = vault.mint.key() == deposit_token.key() @ProgramErrorCode::WrongDepositVault,
    )]
    pub vault: Account<'info, TokenAccount>,

    pub deposit_token: Account<'info, Mint>,

    #[account(
        mut,
        constraint = admin_token_account.owner == admin.key(),
    )]
    pub admin_token_account: Account<'info, TokenAccount>,

    pub admin: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(has_one = admin @ProgramErrorCode::WrongExchangeBoothAdmin)]
    pub exchange_booth: Account<'info, ExchangeBooth>,

    #[account( 
        mut, 
        seeds = [
            match withdraw_token.key() == exchange_booth.token_a {
                true => b"vault_a",
                _ => b"vault_b"
            },  
            exchange_booth.key().as_ref(),
            withdraw_token.key().as_ref(),
            admin.key().as_ref()
        ],
        bump = match withdraw_token.key() == exchange_booth.token_a {
            true => exchange_booth.vault_a_bump,
            _ => exchange_booth.vault_b_bump
        },  
        constraint = vault.mint == withdraw_token.key() @ProgramErrorCode::WrongDepositVault,
    )]
    pub vault: Account<'info, TokenAccount>,

    pub withdraw_token: Account<'info, Mint>,

    #[account(
        mut,
        constraint = recipient_token_account.mint == withdraw_token.key(),
    )]
    pub recipient_token_account: Account<'info, TokenAccount>,

    pub admin: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct Exchange<'info> {
    pub exchange_booth: Account<'info, ExchangeBooth>,

    pub token_a: Account<'info, Mint>,
    pub token_b: Account<'info, Mint>,

    #[account( 
        mut,
        seeds = [
            b"vault_a",
            exchange_booth.key().as_ref(),
            token_a.key().as_ref(),
            exchange_booth.admin.key().as_ref()
        ],
        bump = exchange_booth.vault_a_bump
    )]
    pub vault_a: Box<Account<'info, TokenAccount>>,

    #[account( 
        mut,
        seeds = [
            b"vault_b",
            exchange_booth.key().as_ref(),
            token_b.key().as_ref(),
            exchange_booth.admin.key().as_ref()
        ],
        bump = exchange_booth.vault_b_bump
    )]
    pub vault_b: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub exchanger_send_token_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub exchanger_receive_token_account: Account<'info, TokenAccount>,

    pub exchanger: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

impl<'info> Deposit<'info> {
    pub fn into_transfer_token_to_vault(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.admin_token_account.to_account_info(),
            to: self.vault.to_account_info(),
            authority: self.admin.to_account_info()
        };

        CpiContext::new(self.token_program.to_account_info().clone(), cpi_accounts)
    }
}

impl<'info> Withdraw<'info> {
    pub fn into_withdraw_token_from_vault(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.vault.to_account_info(),
            to: self.recipient_token_account.to_account_info(),
            authority: self.vault.to_account_info()
        };

        CpiContext::new(self.token_program.to_account_info().clone(), cpi_accounts)
    }
}

impl<'info> Exchange<'info> {
    pub fn into_transfer_token_to_vault(&self, from_token_a: bool) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.exchanger_send_token_account.to_account_info(),
            to: match from_token_a {
                true => self.vault_a.to_account_info(),
                _ => self.vault_b.to_account_info()
            },
            authority: self.exchanger.to_account_info()
        };

        CpiContext::new(self.token_program.to_account_info().clone(), cpi_accounts)
    }

    pub fn into_transfer_token_from_vault_to_recipient(&self, from_token_a: bool) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let vault = match from_token_a {
            true => self.vault_a.to_account_info(),
            _ => self.vault_b.to_account_info()
        };

        let cpi_accounts = Transfer {
            from: vault.clone(),
            to: self.exchanger_receive_token_account.to_account_info(),
            authority: vault
        };

        CpiContext::new(self.token_program.to_account_info().clone(), cpi_accounts)
    }
}
