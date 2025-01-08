use anchor_lang::prelude::*;
use anchor_spl::{
    token::{self, Mint, Token},
    token_interface::spl_token_metadata_interface::state::TokenMetadata,
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

    // pub fn create_fungible_token(ctx: Context<CreateFungibleToken>) -> Result<()> {
    //     // Initialize mint with studio PDA as authority
    //     token::initialize_mint(
    //         CpiContext::new(
    //             ctx.accounts.token_program.to_account_info(),
    //             token::InitializeMint {
    //                 mint: ctx.accounts.mint.to_account_info(),
    //                 rent: ctx.accounts.rent.to_account_info(),
    //             },
    //         ),
    //         9, // decimals
    //         &ctx.accounts.pda.key(),
    //         Some(&ctx.accounts.pda.key()),
    //     )?;

    //     // Update the native token field in the PDA with the mint address
    //     let entry_account = &mut ctx.accounts.pda;
    //     entry_account.native_token = ctx.accounts.mint.key();

    //     // Create metadata using Metaplex's generated instruction
    //     let create_metadata_ix = CreateMetadataAccountV3 {
    //         metadata: ctx.accounts.metadata.key(),
    //         mint: ctx.accounts.mint.key(),
    //         mint_authority: ctx.accounts.pda.key(),
    //         payer: ctx.accounts.payer.key(),
    //         update_authority: (ctx.accounts.pda.key(), true),
    //         system_program: ctx.accounts.system_program.key(),
    //         rent: Some(ctx.accounts.rent.key()),
    //     }
    //     .instruction(CreateMetadataAccountV3InstructionArgs {
    //         data: DataV2 {
    //             name: ctx.accounts.pda.name.clone(),
    //             symbol: ctx.accounts.pda.symbol.clone(),
    //             uri: ctx.accounts.pda.uri.clone(),
    //             seller_fee_basis_points: 0,
    //             creators: None,
    //             collection: None,
    //             uses: None,
    //         },
    //         is_mutable: true,
    //         collection_details: None,
    //     });

    //     // Execute the create metadata instruction
    //     invoke_signed(
    //         &create_metadata_ix,
    //         &[
    //             ctx.accounts.metadata.to_account_info(),
    //             ctx.accounts.mint.to_account_info(),
    //             ctx.accounts.pda.to_account_info(),
    //             ctx.accounts.payer.to_account_info(),
    //             ctx.accounts.pda.to_account_info(),
    //             ctx.accounts.system_program.to_account_info(),
    //             ctx.accounts.rent.to_account_info(),
    //         ],
    //         &[&[
    //             b"registry",
    //             ctx.accounts.entry_seed.key().as_ref(),
    //             &[ctx.bumps.pda],
    //         ]],
    //     )?;

    //     msg!("Fungible token created successfully for game studio");
    //     Ok(())
    // }

    // pub fn mint_fungible_token(
    //     ctx: Context<MintFungibleToken>,
    //     amount: u64,
    // ) -> Result<()> {
    //     // Mint tokens to the recipient's token account
    //     token::mint_to(
    //         CpiContext::new_with_signer(
    //             ctx.accounts.token_program.to_account_info(),
    //             token::MintTo {
    //                 mint: ctx.accounts.mint.to_account_info(),
    //                 to: ctx.accounts.recipient_token_account.to_account_info(),
    //                 authority: ctx.accounts.pda.to_account_info(),
    //             },
    //             &[&[
    //                 b"registry",
    //                 ctx.accounts.entry_seed.key().as_ref(),
    //                 &[ctx.bumps.pda],
    //             ]],
    //         ),
    //         amount,
    //     )?;

    //     msg!("Minted {} tokens to recipient", amount);
    //     Ok(())
    // }

    // pub fn create_nft_collection(ctx: Context<CreateNFTCollection>) -> Result<()> {
    //     // Initialize collection mint
    //     token::initialize_mint(
    //         CpiContext::new(
    //             ctx.accounts.token_program.to_account_info(),
    //             token::InitializeMint {
    //                 mint: ctx.accounts.collection_mint.to_account_info(),
    //                 rent: ctx.accounts.rent.to_account_info(),
    //             },
    //         ),
    //         0, // 0 decimals for NFT
    //         &ctx.accounts.pda.key(),
    //         Some(&ctx.accounts.pda.key()),
    //     )?;

    //     // Update the collection field in the PDA
    //     let entry_account = &mut ctx.accounts.pda;
    //     entry_account.nft_collection = ctx.accounts.collection_mint.key();

    //     // Create metadata for collection
    //     let create_metadata_ix = CreateMetadataAccountV3 {
    //         metadata: ctx.accounts.metadata.key(),
    //         mint: ctx.accounts.collection_mint.key(),
    //         mint_authority: ctx.accounts.pda.key(),
    //         payer: ctx.accounts.payer.key(),
    //         update_authority: (ctx.accounts.pda.key(), true),
    //         system_program: ctx.accounts.system_program.key(),
    //         rent: Some(ctx.accounts.rent.key()),
    //     }
    //     .instruction(CreateMetadataAccountV3InstructionArgs {
    //         data: DataV2 {
    //             name: format!("{} Collection", ctx.accounts.pda.name),
    //             symbol: ctx.accounts.pda.symbol.clone(),
    //             uri: ctx.accounts.pda.uri.clone(),
    //             seller_fee_basis_points: 0,
    //             creators: None,
    //             collection: None,
    //             uses: None,
    //         },
    //         is_mutable: true,
    //         collection_details: Some(mpl_token_metadata::types::CollectionDetails::V1 { size: 0 }),
    //     });

    //     // Execute the create metadata instruction
    //     invoke_signed(
    //         &create_metadata_ix,
    //         &[
    //             ctx.accounts.metadata.to_account_info(),
    //             ctx.accounts.collection_mint.to_account_info(),
    //             ctx.accounts.pda.to_account_info(),
    //             ctx.accounts.payer.to_account_info(),
    //             ctx.accounts.pda.to_account_info(),
    //             ctx.accounts.system_program.to_account_info(),
    //             ctx.accounts.rent.to_account_info(),
    //         ],
    //         &[&[
    //             b"registry",
    //             ctx.accounts.entry_seed.key().as_ref(),
    //             &[ctx.bumps.pda],
    //         ]],
    //     )?;

    //     msg!("NFT collection created successfully for game studio");
    //     Ok(())
    // }

    // pub fn mint_nft(
    //     ctx: Context<MintNFT>,
    //     name: String,
    //     symbol: String,
    //     uri: String,
    // ) -> Result<()> {
    //     // Initialize NFT mint
    //     token::initialize_mint(
    //         CpiContext::new(
    //             ctx.accounts.token_program.to_account_info(),
    //             token::InitializeMint {
    //                 mint: ctx.accounts.nft_mint.to_account_info(),
    //                 rent: ctx.accounts.rent.to_account_info(),
    //             },
    //         ),
    //         0, // 0 decimals for NFT
    //         &ctx.accounts.pda.key(),
    //         Some(&ctx.accounts.pda.key()),
    //     )?;

    //     // Create metadata for NFT
    //     let create_metadata_ix = CreateMetadataAccountV3 {
    //         metadata: ctx.accounts.metadata.key(),
    //         mint: ctx.accounts.nft_mint.key(),
    //         mint_authority: ctx.accounts.pda.key(),
    //         payer: ctx.accounts.payer.key(),
    //         update_authority: (ctx.accounts.pda.key(), true),
    //         system_program: ctx.accounts.system_program.key(),
    //         rent: Some(ctx.accounts.rent.key()),
    //     }
    //     .instruction(CreateMetadataAccountV3InstructionArgs {
    //         data: DataV2 {
    //             name,
    //             symbol,
    //             uri,
    //             seller_fee_basis_points: 0,
    //             creators: None,
    //             collection: Some(mpl_token_metadata::types::Collection {
    //                 verified: false, // Will be verified in a separate instruction
    //                 key: ctx.accounts.collection_mint.key(),
    //             }),
    //             uses: None,
    //         },
    //         is_mutable: true,
    //         collection_details: None,
    //     });

    //     // Execute the create metadata instruction
    //     invoke_signed(
    //         &create_metadata_ix,
    //         &[
    //             ctx.accounts.metadata.to_account_info(),
    //             ctx.accounts.nft_mint.to_account_info(),
    //             ctx.accounts.pda.to_account_info(),
    //             ctx.accounts.payer.to_account_info(),
    //             ctx.accounts.pda.to_account_info(),
    //             ctx.accounts.system_program.to_account_info(),
    //             ctx.accounts.rent.to_account_info(),
    //         ],
    //         &[&[
    //             b"registry",
    //             ctx.accounts.entry_seed.key().as_ref(),
    //             &[ctx.bumps.pda],
    //         ]],
    //     )?;

    //     msg!("NFT minted successfully");
    //     Ok(())
    // }
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
    
    /// CHECK: This is the player's token account
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

// #[derive(Accounts)]
// pub struct MintFungibleToken<'info> {
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
//         mut,
//         constraint = mint.key() == pda.native_token @ ErrorCode::InvalidTokenMint
//     )]
//     pub mint: Account<'info, Mint>,

//     /// CHECK: This is safe as it's checked by the token program
//     #[account(mut)]
//     pub recipient_token_account: UncheckedAccount<'info>,

//     /// CHECK: This is safe as we're just using it for seeds
//     pub entry_seed: UncheckedAccount<'info>,

//     pub token_program: Program<'info, Token>,
// }

// Error codes
// #[error_code]
// pub enum ErrorCode {
//     #[msg("Game studio does not exist")]
//     GameStudioNotFound,
//     #[msg("Invalid game studio admin")]
//     InvalidGameStudioAdmin,
//     #[msg("Invalid token metadata program")]
//     InvalidTokenMetadataProgram,
//     #[msg("Invalid collection")]
//     InvalidCollection,
//     #[msg("Invalid token mint")]
//     InvalidTokenMint,
//     #[msg("Creator mismatch")]
//     CreatorMismatch,
// }


