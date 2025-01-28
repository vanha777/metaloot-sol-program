use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::{
    associated_token::get_associated_token_address,
    token::{self, Token},
};

declare_id!("v3MbKaZSQJrwZWUz81cQ3kc8XvMsiNNxZjM3vN5BB32");

#[program]
pub mod metaloot_registry_program {
    use super::*;

    pub fn create_game_studio(
        ctx: Context<CreateGameStudio>,
        name: String,
        symbol: String,
        uri: String,
        authority: Pubkey,
        native_token: Pubkey,
        nft_collection: Vec<Pubkey>,
    ) -> Result<()> {
        // Store the entry data
        let entry_account = &mut ctx.accounts.pda;
        let metadata = GameRegistryMetadata {
            name,
            symbol,
            uri,
            authority,
            native_token,
            nft_collection,
            bump: ctx.bumps.pda,
        };
        entry_account.set_inner(metadata);
        msg!(
            "Game studio created successfully with entry PDA: {}",
            ctx.accounts.pda.key()
        );
        Ok(())
    }

    pub fn update_game_studio(
        ctx: Context<UpdateGameStudio>,
        // update_metadata: GameRegistryMetadata,
        name: String,
        symbol: String,
        uri: String,
        // authority: Pubkey,
        native_token: Pubkey,
        nft_collection: Vec<Pubkey>,
    ) -> Result<()> {
        let entry_account = &mut ctx.accounts.pda;

        // Update fields if provided
        if !name.trim().is_empty() {
            entry_account.name = name;
        }
        if !symbol.trim().is_empty() {
            entry_account.symbol = symbol;
        }
        if !uri.trim().is_empty() {
            entry_account.uri = uri;
        }
        if native_token != Pubkey::default() {
            entry_account.native_token = native_token;
        }

        // if nft_collection != Pubkey::default() {
        //     entry_account.nft_collection = nft_collection;
        // }

        entry_account.nft_collection = nft_collection;

        msg!("Game studio metadata updated successfully");
        Ok(())
    }

    pub fn create_player_account(
        ctx: Context<CreatePlayerAccount>,
        username: String,
        uri: String,
    ) -> Result<()> {
        let player_account = &mut ctx.accounts.player_pda;
        let clock = Clock::get()?;

        player_account.authority = ctx.accounts.payer.key();
        player_account.username = username;
        player_account.uri = uri;
        player_account.created_at = clock.unix_timestamp;
        player_account.bump = ctx.bumps.player_pda;
        msg!("Player account created successfully");
        Ok(())
    }

    pub fn update_player_account(ctx: Context<UpdatePlayerAccount>, new_uri: String) -> Result<()> {
        let player_account = &mut ctx.accounts.player_pda;

        // Update admin if provided
        // if let Some(admin) = new_admin {
        //     player_account.authority = admin;
        // }

        player_account.uri = new_uri;

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

    pub fn transfer_tokens(ctx: Context<TransferTokens>, amount: u64) -> Result<()> {
        let entry_seed_key = ctx.accounts.sender_seed.key();
        // Create seeds array for PDA signing
        let seeds = &[
            b"player",
            entry_seed_key.as_ref(),
            &[ctx.accounts.sender_pda.bump],
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

    pub fn reward_tokens(ctx: Context<RewardTokens>, amount: u64) -> Result<()> {
        let entry_seed_key = ctx.accounts.sender_seed.key();
        // Create seeds array for PDA signing
        let seeds = &[
            b"registry",
            entry_seed_key.as_ref(),
            &[ctx.accounts.sender_pda.bump],
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
    pub authority: Pubkey,
    pub native_token: Pubkey,
    pub nft_collection: Vec<Pubkey>,
    pub bump: u8,
}

#[derive(Accounts)]
#[instruction(
    name: String,
    symbol: String,
    uri: String,
    authority: Pubkey,
    native_token: Pubkey,
    nft_collection: Pubkey,
)]
pub struct CreateGameStudio<'info> {
    #[account(
        mut,
        constraint = payer.key() == authority @ ErrorCode::ConstraintOwner
    )]
    pub payer: Signer<'info>,

    #[account(
        init,
        payer = payer,
        space = 8  // discriminator
            + 4 + 32  // name (String - 4 bytes for length + max 32 bytes for content)
            + 4 + 32  // symbol (String - 4 bytes for length + max 32 bytes for content)
            + 4 + 200 // uri (String - 4 bytes for length + max 200 bytes for content)
            + 32      // authority (Pubkey)
            + 32      // native_token (Pubkey)
            + 4 + (32 * 5) // nft_collection (Vec<Pubkey> - 4 bytes for length + space for 5 Pubkeys) -- before 32 for single Pubkey
            + 1,      // bump (u8)
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
        constraint = payer.key() == pda.authority @ ErrorCode::ConstraintOwner
    )]
    pub payer: Signer<'info>,

    #[account(
        mut,
        seeds = [b"registry", entry_seed.key().as_ref()],
        bump = pda.bump,
        constraint = !pda.symbol.trim().is_empty() @ ErrorCode::ConstraintAccountIsNone
    )]
    pub pda: Account<'info, GameRegistryMetadata>,
    /// CHECK: This is safe as we're just using it as a reference for PDA seeds
    pub entry_seed: AccountInfo<'info>,
}

#[account]
#[derive(Default)]
pub struct PlayerAccount {
    pub authority: Pubkey, // Authority wallet address
    pub username: String,  // Player's username
    pub created_at: i64,   // Timestamp of account creation
    pub uri: String,
    pub bump: u8,
}

#[derive(Accounts)]
#[instruction(username: String, uri: String)]
pub struct CreatePlayerAccount<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        init,
        payer = payer,
        space = 8 + // discriminator
               32 + // authority (Pubkey)
               4 + 64 + // username (String: 4 bytes for length + max 64 bytes for content)
               8 + // created_at (i64)
               4 + 200 + // uri (String: 4 bytes for length + max 200 bytes for content)
               1,  // bump (u8)
        seeds = [
            b"player",
            entry_seed.key().as_ref()
        ],
        bump,
    )]
    pub player_pda: Account<'info, PlayerAccount>,
    /// CHECK: This is safe as we're just using it as a reference for PDA seeds
    pub entry_seed: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdatePlayerAccount<'info> {
    #[account(
        mut,
        constraint = payer.key() == player_pda.authority @ ErrorCode::ConstraintOwner
    )]
    pub payer: Signer<'info>,

    #[account(
        mut,
        seeds = [
            b"player",
            entry_seed.key().as_ref()
        ],
        bump = player_pda.bump,
        constraint = player_pda.created_at != 0 @ ErrorCode::ConstraintAccountIsNone
    )]
    pub player_pda: Account<'info, PlayerAccount>,

    /// CHECK: This is safe as we're just using it as a reference for PDA seeds
    pub entry_seed: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct InitializePlayerTokenAccounts<'info> {
    #[account(
        mut,
        constraint = payer.key() == player_pda.authority @ ErrorCode::ConstraintOwner
    )]
    pub payer: Signer<'info>,

    /// CHECK: This is the token mint address
    pub token_mint: AccountInfo<'info>,

    #[account(
        seeds = [b"player", entry_seed.key().as_ref()],
        bump = player_pda.bump,
        constraint = player_pda.created_at != 0 @ ErrorCode::ConstraintAccountIsNone
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
        constraint = payer.key() == sender_pda.authority @ ErrorCode::ConstraintOwner
    )]
    pub payer: Signer<'info>,

    /// CHECK: This is the token mint address
    pub token_mint: AccountInfo<'info>,

    /// CHECK: This is safe as we're just using it as a reference for PDA seeds
    pub sender_seed: AccountInfo<'info>,

    #[account(
        seeds = [b"player", sender_seed.key().as_ref()],
        bump = sender_pda.bump,
        constraint = sender_pda.created_at != 0 @ ErrorCode::ConstraintAccountIsNone
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
        bump = recipient_pda.bump,
        constraint = recipient_pda.created_at != 0 @ ErrorCode::ConstraintAccountIsNone
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

#[derive(Accounts)]
pub struct RewardTokens<'info> {
    #[account(
        mut,
        constraint = payer.key() == sender_pda.authority @ ErrorCode::ConstraintOwner
    )]
    pub payer: Signer<'info>,

    /// CHECK: This is the token mint address
    pub token_mint: AccountInfo<'info>,

    /// CHECK: This is safe as we're just using it as a reference for PDA seeds
    pub sender_seed: AccountInfo<'info>,

    #[account(
        seeds = [b"registry", sender_seed.key().as_ref()],
        bump = sender_pda.bump,
        constraint = !sender_pda.name.is_empty() @ ErrorCode::ConstraintAccountIsNone
    )]
    pub sender_pda: Account<'info, GameRegistryMetadata>,

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
        bump = recipient_pda.bump,
        constraint = recipient_pda.created_at != 0 @ ErrorCode::ConstraintAccountIsNone
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
