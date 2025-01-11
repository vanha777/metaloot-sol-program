use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::get_associated_token_address, token::{self, Mint, Token}, token_interface::spl_token_metadata_interface::state::TokenMetadata
};
// use mpl_token_metadata::types::{Creator, DataV2};
use anchor_spl::associated_token::AssociatedToken;
use mpl_token_metadata::ID as TOKEN_METADATA_ID;

declare_id!("v3MbKaZSQJrwZWUz81cQ3kc8XvMsiNNxZjM3vN5BB32");

#[program]
pub mod metaloot_registry_program {
    use anchor_lang::solana_program::program::invoke_signed;
    use mpl_token_metadata::{
        instructions::{CreateMetadataAccountV3, CreateMetadataAccountV3InstructionArgs},
        types::DataV2,
    };

    use super::*;

    pub fn create_game_studio(
        ctx: Context<CreateGameStudio>,
        metadata: GameRegistryMetadata,
    ) -> Result<()> {
        // Store the entry data
        let entry_account = &mut ctx.accounts.pda;
        entry_account.set_inner(metadata);
        msg!(
            "Game studio created successfully with entry PDA: {}",
            ctx.accounts.pda.key()
        );
        Ok(())
    }

    pub fn update_game_studio(
        ctx: Context<UpdateGameStudio>,
        update_metadata: GameRegistryMetadata,
    ) -> Result<()> {
        let entry_account = &mut ctx.accounts.pda;

        // Update fields if provided
        if !update_metadata.name.trim().is_empty() {
            entry_account.name = update_metadata.name;
        }
        if !update_metadata.symbol.trim().is_empty() {
            entry_account.symbol = update_metadata.symbol;
        }
        if !update_metadata.uri.trim().is_empty() {
            entry_account.uri = update_metadata.uri;
        }
        if update_metadata.native_token != Pubkey::default() {
            entry_account.native_token = update_metadata.native_token;
        }

        if update_metadata.nft_collection != Pubkey::default() {
            entry_account.nft_collection = update_metadata.nft_collection;
        }

        msg!("Game studio metadata updated successfully");
        Ok(())
    }

    pub fn create_player_account(
        ctx: Context<CreatePlayerAccount>,
        username: String,
    ) -> Result<()> {
        let player_account = &mut ctx.accounts.player_account;
        let clock = Clock::get()?;

        player_account.admin = ctx.accounts.payer.key();
        player_account.username = username;
        player_account.created_at = clock.unix_timestamp;

        msg!("Player account created successfully");
        Ok(())
    }

    pub fn update_player_account(
        ctx: Context<UpdatePlayerAccount>,
        new_username: Option<String>,
        new_admin: Option<Pubkey>,
    ) -> Result<()> {
        let player_account = &mut ctx.accounts.player_account;

        // Update username if provided
        if let Some(username) = new_username {
            player_account.username = username;
        }

        // Update admin if provided
        if let Some(admin) = new_admin {
            player_account.admin = admin;
        }

        msg!("Player account updated successfully");
        Ok(())
    }

    pub fn initialize_player_token_accounts(
        ctx: Context<InitializePlayerTokenAccounts>,
    ) -> Result<()> {
        // Create ATA for the token
        anchor_spl::associated_token::create(CpiContext::new(
            ctx.accounts.associated_token_program.to_account_info(),
            anchor_spl::associated_token::Create {
                payer: ctx.accounts.payer.to_account_info(),
                associated_token: ctx.accounts.player_token_account.to_account_info(),
                authority: ctx.accounts.player_pda.to_account_info(),
                mint: ctx.accounts.token_mint.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                token_program: ctx.accounts.token_program.to_account_info(),
            },
        ))?;
        msg!("Created player token account");

        msg!("Player token account initialized successfully");
        Ok(())
    }

    pub fn transfer_tokens(
        ctx: Context<TransferTokens>,
        amount: u64,
    ) -> Result<()> {
        let entry_seed_key = ctx.accounts.sender_seed.key();
        // Create seeds array for PDA signing
        let seeds = &[
            b"player",
            entry_seed_key.as_ref(),
            &[ctx.bumps.sender_pda],
        ];
        let signer_seeds = &[&seeds[..]];

        // Execute transfer with PDA as signing authority
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                token::Transfer {
                    from: ctx.accounts.sender_token_account.to_account_info(),
                    to: ctx.accounts.recipient_token_account.to_account_info(),
                    authority: ctx.accounts.sender_pda.to_account_info(),
                },
                signer_seeds,
            ),
            amount,
        )?;
        msg!("Transferred {} tokens successfully", amount);
        Ok(())
    }
}

#[account]
#[derive(Default)]
pub struct GameRegistryMetadata {
    pub name: String,
    pub symbol: String,
    pub uri: String,
    pub creator: Pubkey,
    pub native_token: Pubkey,
    pub nft_collection: Pubkey,
}

#[derive(Accounts)]
#[instruction(metadata: GameRegistryMetadata)]
pub struct CreateGameStudio<'info> {
    #[account(
        mut,
        constraint = payer.key() == metadata.creator @ ErrorCode::ConstraintOwner
    )]
    pub payer: Signer<'info>,

    #[account(
        init,
        payer = payer,
        space = 8 
            + 32  // creator (Pubkey)
            + 4 + 32  // name (String - 4 bytes for length + max 32 bytes for content)
            + 4 + 200 // description (String - 4 bytes for length + max 200 bytes for content)
            + 4 + 200, // uri (String - 4 bytes for length + max 200 bytes for content)
        seeds = [b"registry", entry_seed.key().as_ref()],
        bump
    )]
    pub pda: Account<'info, GameRegistryMetadata>,

    /// CHECK: This is safe as we're just using it as a reference for PDA seeds
    pub entry_seed: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateGameStudio<'info> {
    #[account(
        mut,
        constraint = payer.key() == pda.creator @ ErrorCode::ConstraintOwner
    )]
    pub payer: Signer<'info>,

    #[account(
        mut,
        seeds = [b"registry", entry_seed.key().as_ref()],
        bump
    )]
    pub pda: Account<'info, GameRegistryMetadata>,
    /// CHECK: This is safe as we're just using it as a reference for PDA seeds
    pub entry_seed: AccountInfo<'info>,
}

// #[derive(Accounts)]
// pub struct CreateFungibleToken<'info> {
//     #[account(
//         mut,
//         constraint = payer.key() == pda.creator @ ErrorCode::InvalidGameStudioAdmin
//     )]
//     pub payer: Signer<'info>,

//     #[account(
//         mut,
//         seeds = [b"registry", entry_seed.key().as_ref()],
//         bump
//     )]
//     pub pda: Account<'info, GameRegistryMetadata>,

//     // #[account(
//     //     mut,
//     //     seeds = [b"token", entry_seed.key().as_ref()],
//     //     bump
//     // )]
//     // /// CHECK: This is safe as we're just using it for seeds
//     // pub mint: AccountInfo<'info>,
//     /// CHECK: This is safe as we're just using it for seeds
//     #[account(
//         init,
//         payer = payer,
//         space = Mint::LEN,
//         seeds = [b"token", entry_seed.key().as_ref()],
//         bump,
//         owner = token::ID
//     )]
//     pub mint: UncheckedAccount<'info>,

//     /// CHECK: This account is checked in the mpl_token_metadata program
//     #[account(
//         mut,
//         seeds = [
//             b"metadata",
//             token_metadata_program.key().as_ref(),
//             mint.key().as_ref()
//         ],
//         bump,
//         seeds::program = TOKEN_METADATA_ID
//     )]
//     pub metadata: UncheckedAccount<'info>,

//     /// CHECK: Verified using the constant ID
//     #[account(address = TOKEN_METADATA_ID)]
//     pub token_metadata_program: UncheckedAccount<'info>,

//     /// CHECK: This is safe as we're just using it for seeds
//     pub entry_seed: UncheckedAccount<'info>,

//     pub system_program: Program<'info, System>,
//     pub token_program: Program<'info, Token>,
//     pub rent: Sysvar<'info, Rent>,
// }

// #[derive(Accounts)]
// pub struct CreateNFTCollection<'info> {
//     #[account(
//         mut,
//         constraint = payer.key() == pda.creator @ ErrorCode::InvalidGameStudioAdmin
//     )]
//     pub payer: Signer<'info>,

//     #[account(
//         mut,
//         seeds = [b"registry", entry_seed.key().as_ref()],
//         bump
//     )]
//     pub pda: Account<'info, GameRegistryMetadata>,

//     #[account(
//         init,
//         payer = payer,
//         space = Mint::LEN,
//         seeds = [b"collection", entry_seed.key().as_ref()],
//         bump,
//         owner = token::ID
//     )]
//     pub collection_mint: UncheckedAccount<'info>,

//     /// CHECK: This account is checked in the mpl_token_metadata program
//     #[account(
//         mut,
//         seeds = [
//             b"metadata",
//             token_metadata_program.key().as_ref(),
//             /// CHECK: This account is checked in the mpl_token_metadata program
//             collection_mint.key().as_ref()
//         ],
//         bump,
//         seeds::program = TOKEN_METADATA_ID
//     )]
//     pub metadata: UncheckedAccount<'info>,

//     /// CHECK: Verified using the constant ID
//     #[account(address = TOKEN_METADATA_ID)]
//     pub token_metadata_program: UncheckedAccount<'info>,

//     /// CHECK: This is safe as we're just using it for seeds
//     pub entry_seed: UncheckedAccount<'info>,

//     pub system_program: Program<'info, System>,
//     pub token_program: Program<'info, Token>,
//     pub rent: Sysvar<'info, Rent>,
// }

// #[derive(Accounts)]
// pub struct MintNFT<'info> {
//     #[account(
//         mut,
//         constraint = payer.key() == pda.creator @ ErrorCode::InvalidGameStudioAdmin
//     )]
//     pub payer: Signer<'info>,

//     #[account(
//         mut,
//         seeds = [b"registry", entry_seed.key().as_ref()],
//         bump
//     )]
//     pub pda: Account<'info, GameRegistryMetadata>,

//     #[account(
//         init,
//         payer = payer,
//         space = Mint::LEN,
//         owner = token::ID
//     )]
//     pub nft_mint: UncheckedAccount<'info>,

//     /// CHECK: This account is checked in the mpl_token_metadata program
//     #[account(
//         mut,
//         seeds = [
//             b"metadata",
//             token_metadata_program.key().as_ref(),
//             nft_mint.key().as_ref()
//         ],
//         bump,
//         seeds::program = TOKEN_METADATA_ID
//     )]
//     pub metadata: UncheckedAccount<'info>,

//     /// CHECK: This is the collection mint
//     #[account(
//         constraint = collection_mint.key() == pda.nft_collection @ ErrorCode::InvalidCollection
//     )]
//     pub collection_mint: UncheckedAccount<'info>,

//     /// CHECK: Verified using the constant ID
//     #[account(address = TOKEN_METADATA_ID)]
//     pub token_metadata_program: UncheckedAccount<'info>,

//     /// CHECK: This is safe as we're just using it for seeds
//     pub entry_seed: UncheckedAccount<'info>,

//     pub system_program: Program<'info, System>,
//     pub token_program: Program<'info, Token>,
//     pub rent: Sysvar<'info, Rent>,
// }

#[account]
#[derive(Default)]
pub struct PlayerAccount {
    pub admin: Pubkey,   // Player's wallet address
    pub username: String, // Player's username
    pub created_at: i64,  // Timestamp of account creation
}

#[derive(Accounts)]
#[instruction(username: String)]
pub struct CreatePlayerAccount<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        init,
        payer = payer,
        space = 8 + // discriminator
               32 + // player (Pubkey)
               4 + 64 + // username (String: 4 bytes for length + max 64 bytes for content)
               8,  // created_at (i64)
        seeds = [
            b"player",
            entry_seed.key().as_ref()
        ],
        bump,
    )]
    pub player_account: Account<'info, PlayerAccount>,
    /// CHECK: This is safe as we're just using it as a reference for PDA seeds
    pub entry_seed: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdatePlayerAccount<'info> {
    #[account(
        mut,
        constraint = payer.key() == player_account.admin @ ErrorCode::ConstraintOwner
    )]
    pub payer: Signer<'info>,

    #[account(
        mut,
        seeds = [
            b"player",
            entry_seed.key().as_ref()
        ],
        bump
    )]
    pub player_account: Account<'info, PlayerAccount>,

    /// CHECK: This is safe as we're just using it as a reference for PDA seeds
    pub entry_seed: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct InitializePlayerTokenAccounts<'info> {
    #[account(
        mut,
        constraint = payer.key() == player_pda.admin @ ErrorCode::ConstraintOwner
    )]
    pub payer: Signer<'info>,
    
    /// CHECK: This is the token mint address
    pub token_mint: AccountInfo<'info>,

    #[account(
        seeds = [b"player", entry_seed.key().as_ref()],
        bump
    )]
    pub player_pda: Account<'info, PlayerAccount>,
    
    /// CHECK: This is the player's token account
    #[account(mut)]
    pub player_token_account: UncheckedAccount<'info>,

    /// CHECK: This is safe as we're just using it as a reference for PDA seeds
    pub entry_seed: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

#[derive(Accounts)]
pub struct TransferTokens<'info> {
    #[account(
        mut,
        constraint = payer.key() == sender_pda.admin @ ErrorCode::ConstraintOwner
    )]
    pub payer: Signer<'info>,

        
    /// CHECK: This is the token mint address
    pub token_mint: AccountInfo<'info>,
    
    /// CHECK: This is safe as we're just using it as a reference for PDA seeds
    pub sender_seed: AccountInfo<'info>,

    #[account(
        seeds = [b"player", sender_seed.key().as_ref()],
        bump,
    )]
    pub sender_pda: Account<'info, PlayerAccount>,

    /// CHECK: Account validated by token program
    #[account(
        mut,
        constraint = sender_token_account.owner == &token_program.key() @ ErrorCode::ConstraintAssociatedTokenTokenProgram,
        constraint = sender_token_account.key() == get_associated_token_address(&sender_pda.key(), &token_mint.key()) @ ErrorCode::ConstraintAssociatedTokenTokenProgram
    )]
    pub sender_token_account: UncheckedAccount<'info>,

    /// CHECK: This is safe as we're just using it as a reference for PDA seeds
    pub recipient_seed: AccountInfo<'info>,

    #[account(
        seeds = [b"player", recipient_seed.key().as_ref()],
        bump,
    )]
    pub recipient_pda: Account<'info, PlayerAccount>,

    /// CHECK: Account validated by token program
    #[account(
        mut,
        constraint = recipient_token_account.owner == &token_program.key() @ ErrorCode::ConstraintAssociatedTokenTokenProgram,
        constraint = recipient_token_account.key() == get_associated_token_address(&recipient_pda.key(), &token_mint.key()) @ ErrorCode::ConstraintAssociatedTokenTokenProgram
    )]
    pub recipient_token_account: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
}


