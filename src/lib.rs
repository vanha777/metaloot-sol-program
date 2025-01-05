// use borsh::{BorshDeserialize, BorshSerialize};
// use solana_program::{
//     account_info::{next_account_info, AccountInfo},
//     entrypoint,
//     entrypoint::ProgramResult,
//     msg,
//     program_error::ProgramError,
//     pubkey::Pubkey,
// };

// /// Define the type of state stored in accounts
// #[derive(BorshSerialize, BorshDeserialize, Debug)]
// pub struct GreetingAccount {
//     /// number of greetings
//     pub counter: u32,
// }

// // Declare and export the program's entrypoint
// entrypoint!(process_instruction);

// // Program entrypoint's implementation
// pub fn process_instruction(
//     program_id: &Pubkey, // Public key of the account the hello world program was loaded into
//     accounts: &[AccountInfo], // The account to say hello to
//     _instruction_data: &[u8], // Ignored, all helloworld instructions are hellos
// ) -> ProgramResult {
//     msg!("Hello World Rust program entrypoint");

//     // Iterating accounts is safer than indexing
//     let accounts_iter = &mut accounts.iter();

//     // Get the account to say hello to
//     let account = next_account_info(accounts_iter)?;

//     // The account must be owned by the program in order to modify its data
//     if account.owner != program_id {
//         msg!("Greeted account does not have the correct program id");
//         return Err(ProgramError::IncorrectProgramId);
//     }

//     // Increment and store the number of times the account has been greeted
//     let mut greeting_account = GreetingAccount::try_from_slice(&account.data.borrow())?;
//     greeting_account.counter += 1;
//     greeting_account.serialize(&mut *account.data.borrow_mut())?;

//     msg!("Greeted {} time(s)!", greeting_account.counter);

//     Ok(())
// }

use borsh::{to_vec, BorshDeserialize, BorshSerialize};
use mpl_token_metadata::generated::{
    instructions::{CreateMetadataAccountV3, CreateMetadataAccountV3InstructionArgs},
    types::{Creator, DataV2},
};
use solana_program::program::invoke_signed;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program::invoke,
    program_error::ProgramError,
    pubkey::Pubkey,
    system_instruction,
    sysvar::Sysvar,
};
use spl_token;

// At the top with other imports/constants
pub const TOKEN_PROGRAM_ID: Pubkey =
    solana_program::pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct GameRegistryMetadata {
    pub name: String,   // Game name
    pub symbol: String, // Game symbol or short identifier
    pub uri: String,
    pub creator: Pubkey,        // Game admin's public key
    pub native_token: Pubkey,   // Game admin's public key
    pub nft_collection: Pubkey, // Game admin's public key
}

impl GameRegistryMetadata {
    pub fn max_size() -> GameRegistryMetadata {
        GameRegistryMetadata {
            name: "x".repeat(32),
            symbol: "x".repeat(10),
            uri: "x".repeat(200),
            creator: Pubkey::default(),
            native_token: Pubkey::default(),
            nft_collection: Pubkey::default(),
        }
    }
}

// ----------------------------------------------------------
// 1) Instruction Enum
// ----------------------------------------------------------
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum RegistryInstruction {
    /// Initialize the registry account (and set admin).
    // InitializeRegistry {
    //     // No additional fields needed; admin is the signer
    // },

    // Create a new game studio NFT and register it.
    CreateGameStudio(GameRegistryMetadata),

    // Update an existing game studio NFT's metadata.
    UpdateGameStudio(GameRegistryMetadata),

    // Create a new fungible token for the game studio
    CreateFungibleToken(),
}

// ----------------------------------------------------------
// 2) Entrypoint
// ----------------------------------------------------------
entrypoint!(process_instruction);
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    msg!("Instruction data length: {}", instruction_data.len());
    msg!("Raw instruction data (hex): {:02x?}", instruction_data);

    // Try to read the variant index (first byte)
    if !instruction_data.is_empty() {
        msg!("Instruction variant index: {}", instruction_data[0]);
    }

    // Detailed deserialization logging
    let instruction = match RegistryInstruction::try_from_slice(instruction_data) {
        Ok(ix) => ix,
        Err(e) => {
            msg!("Deserialization error: {:?}", e);
            msg!("Make sure client-side enum matches:");
            msg!("0 => InitializeRegistry");
            msg!("1 => CreateGameStudio {{ name: String, symbol: String, uri: String, creator: Pubkey }}");
            msg!("2 => UpdateGameStudio {{ new_uri: Option<String> }}");
            return Err(ProgramError::InvalidInstructionData);
        }
    };

    match instruction {
        // RegistryInstruction::InitializeRegistry {} => {
        //     msg!("Instruction: InitializeRegistry");
        //     initialize_registry(program_id, accounts)
        // }
        RegistryInstruction::CreateGameStudio(metadata) => {
            msg!("Instruction: CreateGameStudio");
            msg!(
                "Name: {}, Symbol: {}, URI: {}, Creator: {}",
                metadata.name,
                metadata.symbol,
                metadata.uri,
                metadata.creator
            );
            create_game_studio(
                program_id,
                accounts,
                metadata.name,
                metadata.symbol,
                metadata.uri,
                metadata.creator,
                metadata.native_token,
                metadata.nft_collection,
            )
        }

        RegistryInstruction::UpdateGameStudio(update_metadata) => {
            msg!("Instruction: UpdateGameStudio");
            update_game_studio(program_id, accounts, update_metadata)
        }

        RegistryInstruction::CreateFungibleToken() => {
            msg!("Instruction: CreateFungibleToken");
            create_fungible_token(program_id, accounts)
        }
    }
}

// ----------------------------------------------------------
// 3) CreateGameStudio Processor
// ----------------------------------------------------------
pub fn create_game_studio(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    name: String,
    symbol: String,
    uri: String,
    creator: Pubkey,
    native_token: Pubkey,
    nft_collection: Pubkey,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    // Get accounts
    let payer_account = next_account_info(account_info_iter)?;
    let system_program = next_account_info(account_info_iter)?;
    let entry_account = next_account_info(account_info_iter)?;
    let mint_account = next_account_info(account_info_iter)?;

    // Verify payer is signer
    if !payer_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    msg!("Creating Registry Entry");

    // Check if entry account already exists
    if !entry_account.data_is_empty() {
        msg!("Entry account already exists");
        return Err(ProgramError::AccountAlreadyInitialized);
    }
    // =======================================================
    // DERIVE THE ENTRY PDA (unique entry key for this game studio)
    // =======================================================
    let entry_seeds = &[b"registry", mint_account.key.as_ref()];
    let (entry_pda, entry_bump) = Pubkey::find_program_address(entry_seeds, program_id);
    // Get the entry account from accounts - this need to be passed in as a parameter
    // Save onchain gas instead of creating a new account
    // Verify entry PDA
    if entry_pda != *entry_account.key {
        return Err(ProgramError::InvalidArgument);
    }

    let max_serialized_size = to_vec(&GameRegistryMetadata::max_size())?.len();
    // Calculate minimum rent-exempt balance
    let rent = solana_program::sysvar::rent::Rent::get()?;
    let entry_lamports = rent.minimum_balance(max_serialized_size);
    // Create the account
    invoke_signed(
        &system_instruction::create_account(
            payer_account.key,
            &entry_pda,
            entry_lamports,
            max_serialized_size as u64,
            program_id,
        ),
        &[
            payer_account.clone(),
            entry_account.clone(),
            system_program.clone(),
        ],
        &[&[b"registry", mint_account.key.as_ref(), &[entry_bump]]],
    )?;
    // Store the entry data in the newly created account
    let entry_data = GameRegistryMetadata {
        name: name.clone(),
        symbol: symbol.clone(),
        uri: uri.clone(),
        creator,
        native_token,
        nft_collection,
    };
    let serialized_entry = to_vec(&entry_data)?;
    entry_account.data.borrow_mut()[..serialized_entry.len()].copy_from_slice(&serialized_entry);

    msg!(
        "Game studio created successfully with entry PDA: {}",
        entry_pda
    );
    Ok(())
}

// ----------------------------------------------------------
// 4) UpdateGameStudio Processor
// ----------------------------------------------------------
pub fn update_game_studio(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    update_metadata: GameRegistryMetadata,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    // Get accounts
    let payer_account = next_account_info(account_info_iter)?;
    let entry_account = next_account_info(account_info_iter)?;
    let mint_account = next_account_info(account_info_iter)?;
    // Verify payer is signer
    if !payer_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Verify the entry PDA matches
    let entry_seeds = &[b"registry", mint_account.key.as_ref()];
    let (entry_pda, _) = Pubkey::find_program_address(entry_seeds, program_id);
    if entry_pda != *entry_account.key {
        return Err(ProgramError::InvalidArgument);
    }
    // Deserialize the existing entry data
    let mut entry_data = GameRegistryMetadata::deserialize(&mut &entry_account.data.borrow()[..])
        .map_err(|_| ProgramError::InvalidAccountData)?;

    // Update the fields if provided
    if !update_metadata.name.trim().is_empty() {
        entry_data.name = update_metadata.name;
    }
    if !update_metadata.symbol.trim().is_empty() {
        entry_data.symbol = update_metadata.symbol;
    }
    if !update_metadata.uri.trim().is_empty() {
        entry_data.uri = update_metadata.uri;
    }

    if update_metadata.native_token != Default::default() {
        entry_data.native_token = update_metadata.native_token;
    }
    if update_metadata.nft_collection != Default::default() {
        entry_data.nft_collection = update_metadata.nft_collection;
    }

    // Serialize and save the updated data
    let serialized_entry = to_vec(&entry_data)?;
    entry_account.data.borrow_mut()[..serialized_entry.len()].copy_from_slice(&serialized_entry);

    msg!("Game studio metadata updated successfully");
    Ok(())
}

// ----------------------------------------------------------
// 5) Create Fungible Token
// ----------------------------------------------------------
pub fn create_fungible_token(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    msg!("debug 0");
    // Get accounts
    let sender_account = next_account_info(account_info_iter)?; // Payer account - Must be admin of the studio
    let mint_account = next_account_info(account_info_iter)?; // Mint account - random keypair
    let entry_account = next_account_info(account_info_iter)?; // Token Mint account
    let game_studio_admin = next_account_info(account_info_iter)?; // Game studio's PDA
    let system_program = next_account_info(account_info_iter)?;
    let token_program = next_account_info(account_info_iter)?;
    let rent_sysvar = next_account_info(account_info_iter)?;

    // Derive game studio's PDA
    let studio_seeds = &[b"registry", entry_account.key.as_ref()];
    let (studio_pda, studio_bump) = Pubkey::find_program_address(studio_seeds, program_id);

    // Verify payer is signer
    if !sender_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    msg!("debug 1");

    // Verify the game studio admin account matches the derived PDA
    if studio_pda != *game_studio_admin.key {
        msg!("Game studio admin account does not match derived PDA");
        return Err(ProgramError::InvalidArgument);
    }

    if game_studio_admin.data_is_empty() {
        msg!("Game studio does not exist");
        return Err(ProgramError::UninitializedAccount);
    }

    // Get rent sysvar
    let rent = &solana_program::sysvar::rent::Rent::from_account_info(rent_sysvar)?;
    msg!("debug 2");
    // Create mint account owned by token program
    let mint_required_lamports = rent.minimum_balance(82);
    invoke(
        &system_instruction::create_account(
            sender_account.key,
            mint_account.key,
            mint_required_lamports,
            82,
            token_program.key, // Important: The owner should be the token program
        ),
        &[
            sender_account.clone(),
            mint_account.clone(),
            system_program.clone(),
        ],
    )?;
    msg!("debug 3");
    // Initialize mint with studio PDA as authority
    invoke_signed(
        &spl_token::instruction::initialize_mint(
            &TOKEN_PROGRAM_ID,
            mint_account.key,
            &studio_pda,       // mint authority
            Some(&studio_pda), // freeze authority
            9,                 // decimals
        )?,
        &[
            mint_account.clone(),
            rent_sysvar.clone(),
        ],
        &[&[b"registry", entry_account.key.as_ref(), &[studio_bump]]],
    )?;

    // Create ATA for recipient (if needed)
    // let recipient_token_account = next_account_info(account_info_iter)?;
    // invoke(
    //     &spl_associated_token_account::instruction::create_associated_token_account(
    //         sender_account.key,    // payer
    //         sender_account.key,    // wallet to create ATA for
    //         mint_account.key,      // mint
    //         &TOKEN_PROGRAM_ID,
    //     ),
    //     &[/* appropriate accounts */],
    // )?;

    // // Mint tokens to the recipient's ATA
    // invoke_signed(
    //     &spl_token::instruction::mint_to(
    //         &TOKEN_PROGRAM_ID,
    //         mint_account.key,
    //         recipient_token_account.key,
    //         &studio_pda,           // mint authority
    //         &[],
    //         1000000000,           // amount (e.g., 1 token with 9 decimals)
    //     )?,
    //     &[
    //         mint_account.clone(),
    //         recipient_token_account.clone(),
    //         game_studio_admin.clone(),
    //     ],
    //     &[&[b"registry", entry_account.key.as_ref(), &[studio_bump]]],
    // )?;

    msg!("Fungible token created and minted successfully for game studio");
    Ok(())
}

// =======================================================
// CREATE TOKENS MINT ACCOUNT (OWNED BY ENTRY PDA)
// =======================================================
// let mint_required_lamports = rent.minimum_balance(82);
// invoke(
//     &system_instruction::create_account(
//         payer_account.key,
//         mint_account.key,
//         mint_required_lamports,
//         82,
//         &TOKEN_PROGRAM_ID,
//     ),
//     &[
//         payer_account.clone(),
//         mint_account.clone(),
//         system_program.clone(),
//     ],
// )?;
// // Initialize mint with entry PDA as authority
// invoke(
//     &spl_token::instruction::initialize_mint(
//         &TOKEN_PROGRAM_ID,
//         mint_account.key,
//         &entry_pda,       // mint authority
//         Some(&entry_pda), // freeze authority
//         0,                // decimals
//     )?,
//     &[mint_account.clone(), rent_sysvar.clone()],
// )?;

// Create metadata account
// ... existing metadata creation code, but with entry_pda as update_authority ...
// let metadata_args = CreateMetadataAccountV3InstructionArgs {
//     data: DataV2 {
//         name,
//         symbol,
//         uri,
//         seller_fee_basis_points: 0,
//         creators: Some(vec![Creator {
//             address: entry_pda, // Changed from registry_pda
//             verified: true,
//             share: 100,
//         }]),
//         collection: None,
//         uses: None,
//     },
//     is_mutable: true,
//     collection_details: None,
// };

// =======================================================
// CREATE ASSOCIATED TOKEN ACCOUNT FOR ENTRY PDA
// =======================================================
// let token_account = next_account_info(account_info_iter)?;
// invoke(
//     &spl_associated_token_account::instruction::create_associated_token_account(
//         payer_account.key,
//         &entry_pda, // token account owner
//         mint_account.key,
//         &TOKEN_PROGRAM_ID,
//     ),
//     &[
//         payer_account.clone(),
//         token_account.clone(),
//         entry_account.clone(),
//         mint_account.clone(),
//         system_program.clone(),
//         token_program.clone(),
//         rent_sysvar.clone(),
//     ],
// )?;

// =======================================================
// MINT 1 TOKEN TO ENTRY PDA'S TOKEN ACCOUNT
// =======================================================
// invoke_signed(
//     &spl_token::instruction::mint_to(
//         &TOKEN_PROGRAM_ID,
//         mint_account.key,
//         token_account.key,
//         &entry_pda,
//         &[],
//         1,
//     )?,
//     &[
//         mint_account.clone(),
//         token_account.clone(),
//         entry_account.clone(),
//     ],
//     &[&[
//         b"registrymetadata",
//         symbol.as_bytes(),
//         name.as_bytes(),
//         &[entry_bump],
//     ]],
// )?;
