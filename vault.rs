use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, MintTo, Burn, Transfer};
use anchor_lang::prelude::*;
use anchor_lang::system_program::{create_account, CreateAccount};
use anchor_spl::{
    token_2022::{
        initialize_mint2,
        spl_token_2022::{
            extension::{
                interest_bearing_mint::InterestBearingConfig, BaseStateWithExtensions,
                ExtensionType, StateWithExtensions,
            },
            pod::PodMint,
            state::Mint as MintState,
        },
        InitializeMint2,
    },
    token_interface::{
        interest_bearing_mint_initialize, interest_bearing_mint_update_rate,
        spl_pod::optional_keys::OptionalNonZeroPubkey, InterestBearingMintInitialize,
        InterestBearingMintUpdateRate, Mint, Token2022,
    },
};
declare_id!("Fg6PaFpoGXkYsidMpWxqSWGoxBWE2D7vhSPLPQFhy6vC");

#[program]
pub mod vault {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, reward_rate: u64) -> Result<()> {
        Ok(())
    }

    pub fn stake(ctx: Context<Stake>, amount: u64) -> Result<()> {
        let staking_data = &mut ctx.accounts.staking_data;

        // Transfer the staked tokens to the vault
        token::transfer(
            ctx.accounts.into_transfer_to_vault_context(),
            amount,
        )?;

        // Mint reward tokens based on staked amount
        let reward_amount = staking_data.reward_rate * amount; // Simple calculation, e.g., 1:1 rate

        token::mint_to(
            ctx.accounts.into_mint_to_context(),
            reward_amount,
        )?;

        Ok(())
    }

    pub fn unstake(ctx: Context<Unstake>, amount: u64) -> Result<()> {
        let staking_data = &mut ctx.accounts.staking_data;

        // Transfer the staked tokens back to the user
        token::transfer(
            ctx.accounts.into_transfer_from_vault_context(),
            amount,
        )?;

        // Burn the reward tokens
        let reward_amount = staking_data.reward_rate * amount; // Simple calculation, e.g., 1:1 rate

        token::burn(
            ctx.accounts.into_burn_context(),
            reward_amount,
        )?;

        Ok(())
    }
}
#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub initializer: Signer<'info>,
    #[account(init, payer = initializer, space = 8 + 32 + 32 + 8)]
    pub staking_data: Account<'info, StakingData>,
    #[account(mut)]
    pub reward_mint: Account<'info, Mint>, // The reward token mint
    #[account(
        init,
        payer = initializer,
        token::mint = reward_mint,
        token::authority = staking_data,
    )]
    pub stake_vault: Account<'info, TokenAccount>, // Vault for staked tokens
    #[account(mut)]
    pub staking_mint: Signer<'info>, // Staking mint authority (must be a signer)
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}


#[derive(Accounts)]
pub struct Stake<'info> {
    #[account(mut)]
    pub staker: Signer<'info>,
    #[account(mut)]
    pub staking_data: Account<'info, StakingData>,
    #[account(mut)]
    pub staker_token_account: Account<'info, TokenAccount>, // User's token account
    #[account(mut)]
    pub stake_vault: Account<'info, TokenAccount>, // Vault for staked tokens
    #[account(mut)]
    pub reward_mint: Account<'info, Mint>, // Mint for the reward tokens
    #[account(mut)]
    pub staker_reward_account: Account<'info, TokenAccount>, // User's reward token account
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct Unstake<'info> {
    #[account(mut)]
    pub staker: Signer<'info>,
    #[account(mut)]
    pub staking_data: Account<'info, StakingData>,
    #[account(mut)]
    pub staker_token_account: Account<'info, TokenAccount>, // User's token account
    #[account(mut)]
    pub stake_vault: Account<'info, TokenAccount>, // Vault for staked tokens
    #[account(mut)]
    pub staker_reward_account: Account<'info, TokenAccount>, // User's reward token account
    #[account(mut)]
    pub reward_mint: Account<'info, Mint>, // Mint for the reward tokens
    pub token_program: Program<'info, Token>,
}

#[account]
pub struct StakingData {
    pub reward_rate: u64,   // How many reward tokens per staked token
    pub stake_vault: Pubkey, // Vault for the staked tokens
    pub reward_mint: Pubkey, // Mint for the reward tokens
}

impl<'info> Stake<'info> {
    fn into_transfer_to_vault_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.staker_token_account.to_account_info(),
            to: self.stake_vault.to_account_info(),
            authority: self.staker.to_account_info(),
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }

    fn into_mint_to_context(&self) -> CpiContext<'_, '_, '_, 'info, MintTo<'info>> {
        let cpi_accounts = MintTo {
            mint: self.reward_mint.to_account_info(),
            to: self.staker_reward_account.to_account_info(),
            authority: self.staking_data.to_account_info(),
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }
}

impl<'info> Unstake<'info> {
    fn into_transfer_from_vault_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.stake_vault.to_account_info(),
            to: self.staker_token_account.to_account_info(),
            authority: self.staking_data.to_account_info(),
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }

    fn into_burn_context(&self) -> CpiContext<'_, '_, '_, 'info, Burn<'info>> {
        let cpi_accounts = Burn {
            mint: self.reward_mint.to_account_info(),
            from: self.stake_vault.to_account_info(),
            authority: self.staker.to_account_info(),
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }
}


// Define the method for account creation context
impl<'info> Initialize<'info> {
    pub fn into_create_account_context(&self) -> CpiContext<'_, '_, '_, 'info, CreateAccount<'info>> {
        let cpi_accounts = CreateAccount {
            payer: self.payer.to_account_info(), // Payer of the transaction
            mint: self.staking_mint.to_account_info(), // New account (mint)
        };
        CpiContext::new(self.system_program.to_account_info(), cpi_accounts)
    }
}