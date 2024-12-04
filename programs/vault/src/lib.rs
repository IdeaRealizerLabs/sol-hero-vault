use anchor_lang::prelude::*;
use anchor_lang::prelude::{Account, AccountSerialize};
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::Token;
use anchor_lang::{ prelude::*, system_program };


use anchor_spl::token_interface::{
     transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked
};
use anchor_lang::prelude::*;
use anchor_spl::metadata::{
        create_master_edition_v3, create_metadata_accounts_v3, sign_metadata, CreateMasterEditionV3,
        CreateMetadataAccountsV3, Metadata, SignMetadata,
    };

use anchor_spl::token_interface::{ mint_to, MintTo };
use mpl_token_metadata::accounts::{ MasterEdition, Metadata as MetadataAccount };

use anchor_lang::solana_program::{keccak, program::invoke_signed};
use anchor_spl::metadata::{ mpl_token_metadata::types::{CollectionDetails, Creator, DataV2, UseMethod, Uses}};
use anchor_spl::metadata::{set_and_verify_sized_collection_item, SetAndVerifySizedCollectionItem};


declare_id!("8nAcwyJcwSPKndVEbrgA2t8JdP7EPNaBN8yDy5m4QCzH");

#[constant]
pub const SEED: &str = "Collection";

const AVATAR_NAMES: [&str; 9] = [
    "GIRL-NINJA",
    "BLUE-NINJA",
    "ROBOT",
    "EVIL-NINJA-1",
    "EVIL-NINJA-2",
    "GHOST",
    "WHITE-NINJA",
    "BLACK-NINJA",
    "SAMURAI",
];

#[error_code]
pub enum ErrorCode {
    #[msg("The staker does not have enough reward mint tokens to burn.")]
    InsufficientRewardMintBalance,
    #[msg("Overflow.")]
    Overflow,
    #[msg("Underflow.")]
    Underflow,
    #[msg("The claim cooldown period has not been completed. Please wait until 24 hours have passed since your last claim.")]
    ClaimCooldownNotFinished,
    #[msg("Wrong stake mint.")]
    WrongStakeMint,
    #[msg("Invalid Mint account space")]
    InvalidMintAccountSpace,
    #[msg("Cant initialize metadata_pointer")]
    CantInitializeMetadataPointer,
}

#[program]
pub mod vault {


    use super::*;

    pub fn init(
        ctx: Context<Initialize>,
        dev_fee: i64,
        owner: Pubkey,
    ) -> Result<()> {
        ctx.accounts.vault_info.owner = owner;
        ctx.accounts.vault_info.total_staked = 0;
        ctx.accounts.vault_info.dev_fee = dev_fee;
        ctx.accounts.vault_info.box_price = 100000 * 10u64.pow(6);
        Ok(())
    }

    pub fn create_collection_nft(
        ctx: Context<CreateCollectionNft>,
        uri: String,
        name: String,
        symbol: String,
    ) -> Result<()> {
        // PDA for signing
        let signer_seeds: &[&[&[u8]]] = &[&[
            SEED.as_bytes(),
            &[ctx.bumps.collection_mint],
        ]];

        // mint collection nft
        mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    mint: ctx.accounts.collection_mint.to_account_info(),
                    to: ctx.accounts.token_account.to_account_info(),
                    authority: ctx.accounts.collection_mint.to_account_info(),
                },
                signer_seeds,
            ),
            1,
        )?;

        // create metadata account for collection nft
        create_metadata_accounts_v3(
            CpiContext::new_with_signer(
                ctx.accounts.token_metadata_program.to_account_info(),
                CreateMetadataAccountsV3 {
                    metadata: ctx.accounts.metadata_account.to_account_info(),
                    mint: ctx.accounts.collection_mint.to_account_info(),
                    mint_authority: ctx.accounts.collection_mint.to_account_info(), // use pda mint address as mint authority
                    update_authority: ctx.accounts.collection_mint.to_account_info(), // use pda mint as update authority
                    payer: ctx.accounts.authority.to_account_info(),
                    system_program: ctx.accounts.system_program.to_account_info(),
                    rent: ctx.accounts.rent.to_account_info(),
                },
                &signer_seeds,
            ),
            DataV2 {
                name: name,
                symbol: symbol,
                uri: uri,
                seller_fee_basis_points: 0,
                creators: Some(vec![Creator {
                    address: ctx.accounts.authority.key(),
                    verified: false,
                    share: 100,
                }]),
                collection: None,
                uses: None,
            },
            true,
            true,
            Some(CollectionDetails::V1 { size: 0 }), // set as collection nft
        )?;

        // create master edition account for collection nft
        create_master_edition_v3(
            CpiContext::new_with_signer(
                ctx.accounts.token_metadata_program.to_account_info(),
                CreateMasterEditionV3 {
                    payer: ctx.accounts.authority.to_account_info(),
                    mint: ctx.accounts.collection_mint.to_account_info(),
                    edition: ctx.accounts.master_edition.to_account_info(),
                    mint_authority: ctx.accounts.collection_mint.to_account_info(),
                    update_authority: ctx.accounts.collection_mint.to_account_info(),
                    metadata: ctx.accounts.metadata_account.to_account_info(),
                    token_program: ctx.accounts.token_program.to_account_info(),
                    system_program: ctx.accounts.system_program.to_account_info(),
                    rent: ctx.accounts.rent.to_account_info(),
                },
                &signer_seeds,
            ),
            Some(0),
        )?;

        // verify creator on metadata account
        sign_metadata(CpiContext::new(
            ctx.accounts.token_metadata_program.to_account_info(),
            SignMetadata {
                creator: ctx.accounts.authority.to_account_info(),
                metadata: ctx.accounts.metadata_account.to_account_info(),
            },
        ))?;
        ctx.accounts.vault_info.collection_mint = ctx.accounts.collection_mint.key();

        Ok(())
    }



    pub fn update_stake_mint(
        ctx: Context<UpdateStakeMint>,
    ) -> Result<()> {
        let signer_key = ctx.accounts.signer.key();
    
        if signer_key != ctx.accounts.vault_info.owner.key() {
            return Err(ProgramError::IllegalOwner.into());
        }
        ctx.accounts.vault_info.stake_mint = ctx.accounts.stake_mint.key();
        Ok(())
    }

    pub fn update_dev_fee(
        ctx: Context<UpdateStakeMint>,
        dev_fee: i64,
    ) -> Result<()> {
        let signer_key = ctx.accounts.signer.key();
    
        if signer_key != ctx.accounts.vault_info.owner.key() {
            return Err(ProgramError::IllegalOwner.into());
        }
        ctx.accounts.vault_info.dev_fee = dev_fee;
        Ok(())
    }

    pub fn transfer_owner(
        ctx: Context<TransferOwner>,
        new_owner: Pubkey,
    ) -> Result<()> {
        let signer_key = ctx.accounts.signer.key();
    
        if signer_key != ctx.accounts.vault_info.owner.key() {
            return Err(ProgramError::IllegalOwner.into());
        }
        ctx.accounts.vault_info.owner = new_owner;
        Ok(())
    }


    // Stake tokens, mint reward tokens, and transfer staked tokens to the vault
    pub fn buy_box(ctx: Context<BuyNFT>) -> Result<()> {

        // let collection_mint = ctx.accounts.collection_mint.key();
    
        // if collection_mint != ctx.accounts.vault_info.collection_mint.key() {
        //     return Err(ProgramError::IllegalOwner.into());
        // }
        // Transfer staked tokens to the vault
        let timestamp = Clock::get()?.unix_timestamp;
    
        msg!("Init metadata {0}", ctx.accounts.vault_info.to_account_info().key);

        let seed = timestamp.to_be_bytes();
        let random_hash = keccak::hash(&seed);
        let random_index = (random_hash.0[0] as usize) % AVATAR_NAMES.len();
        let name = AVATAR_NAMES[random_index];

        // Base URI cố định

      
        let timestamp = Clock::get()?.unix_timestamp; // Lấy timestamp hiện tại

        // Chuyển đổi timestamp thành byte array để làm seed
        let seed = timestamp.to_be_bytes();

        // Tạo hash từ seed bằng cách sử dụng công thức như đã đề cập
        let timestamp_random = seed.iter().fold(0u64, |acc, &byte| acc.wrapping_add(byte as u64));

        let hash = timestamp_random.wrapping_mul(6364136223846793005).wrapping_add(1);

        let random_number = (hash % 100) as u8;

        let rarity = if random_number < 70 {
            1
        } else if random_number < 90 {
            2
        } else {
            3
        };
        let interest_rate = if rarity == 1 {
            1825
        } else if rarity == 2 {
            2920
        } else {
            3650
        };
        let base_uri = if random_number < 70 {
            "https://bafybeifgvugqgl72xxztucdsv2l3x6fl67wksogqdrggnmomscqm5xte4u.ipfs.w3s.link/"
        } else if random_number < 90 {
             "https://bafybeia3hjnbr3jhmsidlp226pffotky5xuhmbj4waotmm5owohcygs6nm.ipfs.w3s.link/"
        } else {
             "https://bafybeigoivokvnbgv6aib27k3g76kx562xmqsksgk7mpusqb4abqicscdy.ipfs.w3s.link/"
        };

        // Tạo URI hoàn chỉnh bằng cách nối chuỗi
        let uri = format!("{}{}.json", base_uri, name); // Ví dụ: https://.../BLACK-NINJA.png

        let signer_seeds: &[&[&[u8]]] = &[&[
            SEED.as_bytes(),
            &[ctx.bumps.collection_mint],
        ]];

        // mint nft in collection
        mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    mint: ctx.accounts.nft_mint.to_account_info(),
                    to: ctx.accounts.token_account.to_account_info(),
                    authority: ctx.accounts.collection_mint.to_account_info(),
                },
                signer_seeds,
            ),
            1,
        )?;

        // create metadata account for nft in collection
        create_metadata_accounts_v3(
            CpiContext::new_with_signer(
                ctx.accounts.token_metadata_program.to_account_info(),
                CreateMetadataAccountsV3 {
                    metadata: ctx.accounts.metadata_account.to_account_info(),
                    mint: ctx.accounts.nft_mint.to_account_info(),
                    mint_authority: ctx.accounts.collection_mint.to_account_info(),
                    update_authority: ctx.accounts.collection_mint.to_account_info(),
                    payer: ctx.accounts.staker.to_account_info(),
                    system_program: ctx.accounts.system_program.to_account_info(),
                    rent: ctx.accounts.rent.to_account_info(),
                },
                &signer_seeds,
            ),
            DataV2 {
                name: "HERO".to_string(),
                symbol: "HERO".to_string(),
                uri: uri,
                seller_fee_basis_points: rarity,
                creators: None,
                collection: None,
                uses: None,
            },
            true,
            true,
            None,
        )?;

        // create master edition account for nft in collection
        create_master_edition_v3(
            CpiContext::new_with_signer(
                ctx.accounts.token_metadata_program.to_account_info(),
                CreateMasterEditionV3 {
                    payer: ctx.accounts.staker.to_account_info(),
                    mint: ctx.accounts.nft_mint.to_account_info(),
                    edition: ctx.accounts.master_edition.to_account_info(),
                    mint_authority: ctx.accounts.collection_mint.to_account_info(),
                    update_authority: ctx.accounts.collection_mint.to_account_info(),
                    metadata: ctx.accounts.metadata_account.to_account_info(),
                    token_program: ctx.accounts.token_program.to_account_info(),
                    system_program: ctx.accounts.system_program.to_account_info(),
                    rent: ctx.accounts.rent.to_account_info(),
                },
                &signer_seeds,
            ),
            Some(0),
        )?;

        // verify nft as part of collection
        set_and_verify_sized_collection_item(
            CpiContext::new_with_signer(
                ctx.accounts.token_metadata_program.to_account_info(),
                SetAndVerifySizedCollectionItem {
                    metadata: ctx.accounts.metadata_account.to_account_info(),
                    collection_authority: ctx.accounts.collection_mint.to_account_info(),
                    payer: ctx.accounts.staker.to_account_info(),
                    update_authority: ctx.accounts.collection_mint.to_account_info(),
                    collection_mint: ctx.accounts.collection_mint.to_account_info(),
                    collection_metadata: ctx.accounts.collection_metadata_account.to_account_info(),
                    collection_master_edition: ctx
                        .accounts
                        .collection_master_edition
                        .to_account_info(),
                },
                &signer_seeds,
            ),
            None,
        )?;

        

        if ctx.accounts.stake_mint.key() != ctx.accounts.vault_info.stake_mint {
            return Err(ErrorCode::WrongStakeMint.into());
        }

        let stake_mint_account: &InterfaceAccount<'_, Mint> = &ctx.accounts.stake_mint;

        transfer_checked(
            ctx.accounts.into_transfer_to_vault_context(),
            ctx.accounts.vault_info.box_price,
            stake_mint_account.decimals,
        )?; 

     
        let dev_fee_percent = 100_i64.checked_sub(ctx.accounts.vault_info.dev_fee).unwrap();
        let elapsed_time = timestamp.checked_sub(ctx.accounts.user_info.last_update).unwrap();

      
        let remaining_reward = elapsed_time
            .checked_mul(ctx.accounts.user_info.interest_rate.try_into().unwrap())
            .and_then(|v| v.checked_div(100))   
            .and_then(|v| v.checked_mul(ctx.accounts.user_info.amount_stake.try_into().unwrap()))
            .and_then(|v| v.checked_div(31536000))
            .and_then(|v| v.checked_mul(dev_fee_percent))
            .and_then(|v| v.checked_add(ctx.accounts.user_info.amount_unprocessed.try_into().unwrap()))
            .and_then(|v| v.checked_div(100))
            .unwrap();

        msg!("timestamp {}",elapsed_time);
        msg!("remaining_reward {}",remaining_reward);
        let interest_rate_final: u64 = (
            (ctx.accounts.user_info.interest_rate * ctx.accounts.user_info.amount_stake) + (interest_rate * ctx.accounts.vault_info.box_price)
          ) / (ctx.accounts.user_info.amount_stake + ctx.accounts.vault_info.box_price);
                  
        ctx.accounts.user_info.amount_stake += ctx.accounts.vault_info.box_price;
        ctx.accounts.user_info.interest_rate = interest_rate_final;
        ctx.accounts.user_info.amount_unprocessed = ctx.accounts.user_info.amount_unprocessed.checked_add(remaining_reward.try_into()?).unwrap();
        ctx.accounts.user_info.last_update = timestamp;
        ctx.accounts.user_info.ref_wallet = ctx.accounts.ref_wallet.key();

        ctx.accounts.vault_info.total_staked = ctx.accounts.vault_info.total_staked
        .checked_add(ctx.accounts.vault_info.box_price)
        .ok_or(ErrorCode::Overflow)?;

        Ok(())
    }



    // Unstake tokens, burn reward tokens, and transfer staked tokens back to user
    pub fn withdraw_lp(ctx: Context<WithdrawLP>) -> Result<()> {
        let stake_mint_account: &InterfaceAccount<'_, Mint> = &ctx.accounts.stake_mint;
        if ctx.accounts.stake_mint.key() != ctx.accounts.vault_info.stake_mint {
            return Err(ErrorCode::WrongStakeMint.into());
        }

        let signer_key = ctx.accounts.signer.key();
    
        if signer_key != ctx.accounts.vault_info.owner.key() {
            return Err(ProgramError::IllegalOwner.into());
        }

        let stake_vault_account: &InterfaceAccount<'_, TokenAccount> = &ctx.accounts.stake_vault_account;

        // 3. Calculate how many reward_mint tokens to mint for the user
        // The formula: mint_amount = (amount staked * total reward mint balance) / stake_mint balance
        let signer_seeds: &[&[&[u8]]] = &[&[b"vault", &[ctx.bumps.vault_info]]];
        // Transfer mUSD tokens to user
        transfer_checked(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                TransferChecked {
                    from: ctx.accounts.stake_vault_account.to_account_info(),
                    mint: ctx.accounts.stake_mint.to_account_info(),
                    to: ctx.accounts.signer_stake_account.to_account_info(),
                    authority: ctx.accounts.vault_info.to_account_info(),
                },
            )
            .with_signer(signer_seeds),
            stake_vault_account.amount as u64,
            stake_mint_account.decimals,
        )?;
        Ok(())
    }

    // Unstake tokens, burn reward tokens, and transfer staked tokens back to user
    pub fn claim(ctx: Context<Claim>) -> Result<()> {
        let stake_mint_account: &InterfaceAccount<'_, Mint> = &ctx.accounts.stake_mint;
        if ctx.accounts.stake_mint.key() != ctx.accounts.vault_info.stake_mint {
            return Err(ErrorCode::WrongStakeMint.into());
        }

        let timestamp = Clock::get()?.unix_timestamp;

       

        let dev_fee_percent = 100_i64.checked_sub(ctx.accounts.vault_info.dev_fee).unwrap();
        let elapsed_time = timestamp.checked_sub(ctx.accounts.user_info.last_update).unwrap();
       
        let remaining_reward = elapsed_time
            .checked_mul(ctx.accounts.user_info.interest_rate.try_into().unwrap())
            .and_then(|v| v.checked_div(100))   
            .and_then(|v| v.checked_mul(ctx.accounts.user_info.amount_stake.try_into().unwrap()))
            .and_then(|v| v.checked_div(31536000))
            .and_then(|v| v.checked_mul(dev_fee_percent))
            .and_then(|v| v.checked_div(100))
            .and_then(|v| v.checked_add(ctx.accounts.user_info.amount_unprocessed.try_into().unwrap()))
            .unwrap();
        msg!("timestamp {}",elapsed_time);
        msg!("remaining_reward {}",remaining_reward);


        // 3. Calculate how many reward_mint tokens to mint for the user
        // The formula: mint_amount = (amount staked * total reward mint balance) / stake_mint balance
        let signer_seeds: &[&[&[u8]]] = &[&[b"vault", &[ctx.bumps.vault_info]]];
        // Transfer mUSD tokens to user
        transfer_checked(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                TransferChecked {
                    from: ctx.accounts.stake_vault_account.to_account_info(),
                    mint: ctx.accounts.stake_mint.to_account_info(),
                    to: ctx.accounts.staker_token_account.to_account_info(),
                    authority: ctx.accounts.vault_info.to_account_info(),
                },
            )
            .with_signer(signer_seeds),
            remaining_reward as u64,
            stake_mint_account.decimals,
        )?;
        ctx.accounts.user_info.last_update=timestamp;
        ctx.accounts.user_info.amount_unprocessed=0;

        let ref_token_account = ctx.accounts.ref_stake_account.to_account_info();
        if ref_token_account.key() != ctx.accounts.staker_token_account.key() {
            let ref_amount = remaining_reward.checked_mul(10).and_then(|v| v.checked_div(100)).unwrap();
            // Transfer mUSD tokens to user
            transfer_checked(
                CpiContext::new(
                    ctx.accounts.token_program.to_account_info(),
                    TransferChecked {
                        from: ctx.accounts.stake_vault_account.to_account_info(),
                        mint: ctx.accounts.stake_mint.to_account_info(),
                        to: ctx.accounts.ref_stake_account.to_account_info(),
                        authority: ctx.accounts.vault_info.to_account_info(),
                    },
                )
                .with_signer(signer_seeds),
                ref_amount as u64,
                stake_mint_account.decimals,
            )?;
        }
        Ok(())
    }


    pub fn withdraw_dev_fee(
        ctx: Context<WithdrawDevFee>,
    ) -> Result<()> {
        let stake_mint_account: &InterfaceAccount<'_, Mint> = &ctx.accounts.stake_mint;

        // Ensure only the authorized mint authority can update the rate
        let signer_key = ctx.accounts.signer.key();
    
        if signer_key != ctx.accounts.vault_info.owner.key() {
            return Err(ProgramError::IllegalOwner.into());
        }
        let total_fee = ctx.accounts.vault_info.total_fee_collect;
        let signer_seeds: &[&[&[u8]]] = &[&[b"vault", &[ctx.bumps.vault_info]]];
        // Transfer mUSD tokens to user
        transfer_checked(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                TransferChecked {
                    from: ctx.accounts.stake_vault_account.to_account_info(),
                    mint: ctx.accounts.stake_mint.to_account_info(),
                    to: ctx.accounts.owner_token_acount.to_account_info(),
                    authority: ctx.accounts.vault_info.to_account_info(),
                },
            )
            .with_signer(signer_seeds),
            total_fee,
            stake_mint_account.decimals,
        )?;
        ctx.accounts.vault_info.total_fee_collect = 0;
        Ok(())
    }

    pub fn get_supply(ctx: Context<GetCurrentRate>) -> Result<u64> {
        Ok(ctx.accounts.vault_info.total_staked)
    }

    pub fn estimate_accrured_interest(ctx: Context<EstimateAccruedInterest>) -> Result<i64> {
        let timestamp = Clock::get()?.unix_timestamp;
        let dev_fee_percent = 100_i64.checked_sub(ctx.accounts.vault_info.dev_fee).unwrap();
        let elapsed_time = timestamp.checked_sub(ctx.accounts.user_info.last_update).unwrap();
        
        let remaining_reward = elapsed_time
            .checked_mul(ctx.accounts.user_info.interest_rate.try_into().unwrap())
            .and_then(|v| v.checked_div(100))   
            .and_then(|v| v.checked_mul(ctx.accounts.user_info.amount_stake.try_into().unwrap()))
            .and_then(|v| v.checked_div(31536000))
            .and_then(|v| v.checked_mul(dev_fee_percent))
            .and_then(|v| v.checked_add(ctx.accounts.user_info.amount_unprocessed.try_into().unwrap()))
            .and_then(|v| v.checked_div(100))
            .unwrap();
        Ok(remaining_reward)
    }
}


// Contexts for the functions
#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        init_if_needed,
        seeds = [b"vault"], 
        bump,
        payer = authority,
        space = 8 + VaultInfo::INIT_SPACE,
    )]
    pub vault_info: Account<'info, VaultInfo>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct CreateCollectionNft<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        init_if_needed,
        seeds = [SEED.as_bytes()],
        bump,
        payer = authority,
        mint::decimals = 0,
        mint::authority = collection_mint,
        mint::freeze_authority = collection_mint
    )]
    pub collection_mint: InterfaceAccount<'info, Mint>,

    /// CHECK:
    #[account(
        mut,
        address=MetadataAccount::find_pda(&collection_mint.key()).0
    )]
    pub metadata_account: UncheckedAccount<'info>,

    /// CHECK:
    #[account(
        mut,
        address=MasterEdition::find_pda(&collection_mint.key()).0
    )]
    pub master_edition: UncheckedAccount<'info>,

    #[account(
        init_if_needed,
        payer = authority,
        associated_token::mint = collection_mint,
        associated_token::authority = authority
    )]
    pub token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(mut,seeds = [b"vault"], bump)]
    pub vault_info: Account<'info, VaultInfo>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_metadata_program: Program<'info, Metadata>,
    pub rent: Sysvar<'info, Rent>,
}


#[derive(Accounts)]
pub struct BuyNFT<'info> {
    #[account(mut)]
    pub staker: Signer<'info>,
    #[account(mut)]
    pub nft_mint: Signer<'info>,
    // Account 1: user_info
    #[account(
        init_if_needed,
        payer = staker,
        seeds = [b"account", staker.key().as_ref()], 
        bump,
        space = 8 + UserInfo::INIT_SPACE,
    )]
    pub user_info: Account<'info, UserInfo>,

    #[account(mut)]
    pub token_account: InterfaceAccount<'info, TokenAccount>, // User's token account

    /// CHECK
    #[account(mut)]
    pub ref_wallet: UncheckedAccount<'info>,
    #[account(mut)]
    pub staker_token_account: InterfaceAccount<'info, TokenAccount>, // User's token account
    #[account(mut)]
    pub stake_mint: InterfaceAccount<'info, Mint>, // Mint for the reward tokens
    #[account(
        mut,
        associated_token::mint = vault_info.stake_mint,
        associated_token::authority = vault_info,
        associated_token::token_program = token_program
    )]
    pub stake_vault_account: InterfaceAccount<'info, TokenAccount>,

    #[account(mut,seeds = [b"vault"], bump)]
    pub vault_info: Account<'info, VaultInfo>,

    #[account(
        mut,
        seeds = [SEED.as_bytes()],
        bump,
    )]
    pub collection_mint: InterfaceAccount<'info, Mint>,

    /// CHECK:
    #[account(
        mut,
        address=MetadataAccount::find_pda(&collection_mint.key()).0
    )]
    pub collection_metadata_account: UncheckedAccount<'info>,

    /// CHECK:
    #[account(
        mut,
        address=MasterEdition::find_pda(&collection_mint.key()).0
    )]
    pub collection_master_edition: UncheckedAccount<'info>,

    /// CHECK:
    #[account(
        mut,
        address=MetadataAccount::find_pda(&nft_mint.key()).0
    )]
    pub metadata_account: UncheckedAccount<'info>,

    /// CHECK:
    #[account(
        mut,
        address= MasterEdition::find_pda(&nft_mint.key()).0
    )]
    pub master_edition: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_metadata_program: Program<'info, Metadata>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
}



#[derive(Accounts)]
pub struct UpdateStakeMint<'info> {
    #[account(mut)]
    pub signer: Signer<'info>, // Payer of the transaction
    #[account(mut)]
    pub stake_mint: InterfaceAccount<'info, Mint>, // The staking token mint // USDC

    #[account(mut,seeds = [b"vault"], bump)]
    pub vault_info: Account<'info, VaultInfo>,

    #[account(
        init_if_needed,
        payer = signer,
        associated_token::mint = stake_mint,
        associated_token::authority = vault_info,
        associated_token::token_program = stake_token_program
    )]
    pub stake_vault_account: InterfaceAccount<'info, TokenAccount>,
 

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub stake_token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}
#[derive(Accounts)]
pub struct TransferOwner<'info> {
    #[account(mut)]
    pub signer: Signer<'info>, // Payer of the transaction
    #[account(mut, seeds = [b"vault"], bump)]
    pub vault_info: Account<'info, VaultInfo>,
}




#[derive(Accounts)]
pub struct UpdateInterest<'info> {
    #[account(signer)]
    pub signer: Signer<'info>, // The signer you want to check
    #[account(mut, seeds = [b"vault"], bump)]
    pub vault_info: Account<'info, VaultInfo>,
}

#[derive(Accounts)]
pub struct WithdrawDevFee<'info> {
    #[account(signer)]
    pub signer: Signer<'info>, // The signer you want to check
    #[account(mut)]
    owner_token_acount: InterfaceAccount<'info, TokenAccount>,
    #[account(mut)]
    pub stake_mint: InterfaceAccount<'info, Mint>, // The reward token mint (updated to include interest-bearing config)
    #[account(
        mut,
        associated_token::mint = stake_mint,
        associated_token::authority = vault_info,
        associated_token::token_program = token_program
    )]
    pub stake_vault_account: InterfaceAccount<'info, TokenAccount>, // User's reward token account    /// CHECK
    #[account(mut, seeds = [b"vault"], bump)]
    pub vault_info: Account<'info, VaultInfo>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct EstimateAccruedInterest<'info> {
    pub vault_info: Account<'info, VaultInfo>,
    pub user_info: Account<'info, UserInfo>,
    pub token_program: Program<'info, Token>,

}

#[derive(Accounts)]
pub struct GetCurrentRate<'info> {
    pub vault_info: Account<'info, VaultInfo>,
    pub token_program: Program<'info, Token>,
}

impl<'info> BuyNFT<'info> {
    pub fn into_transfer_to_vault_context(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, TransferChecked<'info>> {
        let cpi_accounts = TransferChecked {
            from: self.staker_token_account.to_account_info(),
            mint: self.stake_mint.to_account_info(),
            to: self.stake_vault_account.to_account_info(),
            authority: self.staker.to_account_info(),
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }
}

#[derive(Accounts)]
pub struct Claim<'info> {
    #[account(mut)]
    pub staker: Signer<'info>,
    #[account(mut)]
    pub staker_token_account: InterfaceAccount<'info, TokenAccount>, // User's token account
    #[account(
        mut,
        associated_token::mint = stake_mint,
        associated_token::authority = vault_info,
        associated_token::token_program = token_program
    )]
    pub stake_vault_account: InterfaceAccount<'info, TokenAccount>, // User's reward token account
    #[account(
        mut,
        associated_token::mint = stake_mint,
        associated_token::authority = user_info.ref_wallet,
        associated_token::token_program = token_program
    )]
    pub ref_stake_account: InterfaceAccount<'info, TokenAccount>, // User's reward token account
    #[account(mut)]
    pub stake_mint: InterfaceAccount<'info, Mint>, // Mint for the reward tokens
    #[account(mut,seeds = [b"vault"], bump)]
    pub vault_info: Account<'info, VaultInfo>,
    #[account(
        mut,
        seeds = [b"account", staker.key().as_ref()], 
        bump,
    )]
    pub user_info: Account<'info, UserInfo>,
    pub token_program: Program<'info, Token>,
}


#[derive(Accounts)]
pub struct WithdrawLP<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        mut,
        associated_token::mint = stake_mint,
        associated_token::authority = vault_info,
        associated_token::token_program = token_program
    )]
    pub stake_vault_account: InterfaceAccount<'info, TokenAccount>, // User's reward token account
    #[account(
        mut,
        associated_token::mint = stake_mint,
        associated_token::authority = signer,
        associated_token::token_program = token_program
    )]
    pub signer_stake_account: InterfaceAccount<'info, TokenAccount>, // User's reward token account
    #[account(mut)]
    pub stake_mint: InterfaceAccount<'info, Mint>, // Mint for the reward tokens
    #[account(mut,seeds = [b"vault"], bump)]
    pub vault_info: Account<'info, VaultInfo>,
    pub token_program: Program<'info, Token>,
}


#[derive(Accounts)]
pub struct Compound<'info> {
    #[account(mut)]
    pub staker: Signer<'info>,
    #[account(mut,seeds = [b"vault"], bump)]
    pub vault_info: Account<'info, VaultInfo>,
    #[account(
        mut,
        seeds = [b"account", staker.key().as_ref()], 
        bump,
    )]
    pub user_info: Account<'info, UserInfo>,
    pub stake_token_program: Program<'info, Token>,
}

#[account]
#[derive(InitSpace)] // automatically calculate the space required for the struct
pub struct UserInfo {
    pub amount_stake: u64,
    pub amount_unprocessed: u64,
    pub last_update: i64,
    pub ref_wallet: Pubkey,
    pub interest_rate: u64,
}


#[account]
#[derive(InitSpace)] // automatically calculate the space required for the struct
pub struct VaultInfo {
    pub dev_fee: i64,
    pub total_staked:  u64,
    pub owner: Pubkey,
    pub collection_mint: Pubkey,
    pub stake_mint: Pubkey,
    pub total_fee_collect: u64,
    pub box_price: u64
}

