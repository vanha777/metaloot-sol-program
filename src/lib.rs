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

use borsh::{BorshDeserialize, BorshSerialize};
use mpl_token_metadata::generated::{
    instructions::{CreateMetadataAccountV3, CreateMetadataAccountV3InstructionArgs},
    types::{Creator, DataV2},
};
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

// ----------------------------------------------------------
// 1) Instruction Enum
// ----------------------------------------------------------
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum RegistryInstruction {
    /// Initialize the registry account (and set admin).
    InitializeRegistry {
        // No additional fields needed; admin is the signer
    },

    /// Create a new game studio NFT and register it.
    CreateGameStudio {
        name: String,   // Name of the game studio
        symbol: String, // Symbol for the NFT
        uri: String,    // URI pointing to off-chain JSON metadata
    },

    /// Update an existing game studio NFT's metadata.
    UpdateGameStudio {
        new_uri: Option<String>, // New URI for updated metadata
    },
}

// ----------------------------------------------------------
// 2) On-chain Data Structure
// ----------------------------------------------------------
#[derive(BorshSerialize, BorshDeserialize, Debug, Default)]
pub struct RegistryData {
    pub is_initialized: bool, // Flag to check if the registry is initialized
    pub admin: Pubkey,        // Admin authority
    pub game_studios: Vec<Pubkey>, // List of game studio Mint Pubkeys (NFTs)
}

// ----------------------------------------------------------
// 3) Entrypoint
// ----------------------------------------------------------
entrypoint!(process_instruction);
pub fn process_instruction(
    program_id: &Pubkey,      // Public key of the account the program was loaded into
    accounts: &[AccountInfo], // Accounts involved in the instruction
    instruction_data: &[u8],  // Serialized instruction data
) -> ProgramResult {
    // Deserialize the incoming instruction
    let instruction = RegistryInstruction::try_from_slice(instruction_data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    match instruction {
        RegistryInstruction::InitializeRegistry {} => {
            msg!("Instruction: InitializeRegistry");
            initialize_registry(program_id, accounts)
        }

        RegistryInstruction::CreateGameStudio { name, symbol, uri } => {
            msg!("Instruction: CreateGameStudio");
            create_game_studio(program_id, accounts, name, symbol, uri)
        }

        RegistryInstruction::UpdateGameStudio { new_uri } => {
            msg!("Instruction: UpdateGameStudio");
            update_game_studio(program_id, accounts, new_uri)
        }
    }
}

// ----------------------------------------------------------
// 4) InitializeRegistry Processor
// ----------------------------------------------------------
pub fn initialize_registry(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    // 1. Registry account (PDA)
    let registry_account = next_account_info(account_info_iter)?;

    // 2. Admin account (must be signer)
    let admin_account = next_account_info(account_info_iter)?;

    // 3. System program
    let system_program = next_account_info(account_info_iter)?;

    // Verify that the admin is a signer
    if !admin_account.is_signer {
        msg!("Admin must be a signer");
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Check if the registry account is already initialized
    let mut registry_data = if registry_account.data.borrow().iter().all(|&x| x == 0) {
        // Account is uninitialized; initialize it
        RegistryData::default()
    } else {
        // Deserialize existing data
        RegistryData::try_from_slice(&registry_account.data.borrow())
            .map_err(|_| ProgramError::InvalidAccountData)?
    };

    if registry_data.is_initialized {
        msg!("Registry account is already initialized");
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    // Set initial data
    registry_data.is_initialized = true;
    registry_data.admin = *admin_account.key;
    registry_data.game_studios = Vec::new();

    // Serialize the data back into the account
    registry_data.serialize(&mut &mut registry_account.data.borrow_mut()[..])?;

    msg!("Registry initialized successfully");
    Ok(())
}

// ----------------------------------------------------------
// 5) CreateGameStudio Processor
// ----------------------------------------------------------
pub fn create_game_studio(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    name: String,
    symbol: String,
    uri: String,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    // Get required accounts
    let metadata_account = next_account_info(account_info_iter)?;
    let mint_account = next_account_info(account_info_iter)?;
    let payer_account = next_account_info(account_info_iter)?;
    let system_program = next_account_info(account_info_iter)?;
    let rent_sysvar = next_account_info(account_info_iter)?;

    // Create metadata args
    let metadata_args = CreateMetadataAccountV3InstructionArgs {
        data: DataV2 {
            name,
            symbol,
            uri,
            seller_fee_basis_points: 0,
            creators: Some(vec![Creator {
                address: *payer_account.key,
                verified: true,
                share: 100,
            }]),
            collection: None,
            uses: None,
        },
        is_mutable: true,
        collection_details: None,
    };

    // Create metadata instruction
    let create_metadata_ix = CreateMetadataAccountV3 {
        metadata: *metadata_account.key,
        mint: *mint_account.key,
        mint_authority: *payer_account.key,
        payer: *payer_account.key,
        update_authority: (*payer_account.key, true),
        system_program: solana_program::system_program::id(),
        rent: Some(solana_program::sysvar::rent::id()),
    }
    .instruction(metadata_args);

    // Invoke the instruction
    invoke(
        &create_metadata_ix,
        &[
            metadata_account.clone(),
            mint_account.clone(),
            payer_account.clone(),
            system_program.clone(),
            rent_sysvar.clone(),
        ],
    )?;

    msg!("Metadata account created successfully");
    Ok(())
}

// ----------------------------------------------------------
// 6) UpdateGameStudio Processor
// ----------------------------------------------------------
pub fn update_game_studio(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    new_uri: Option<String>,
) -> ProgramResult {
    // Create an iterator over the accounts
    let account_info_iter = &mut accounts.iter();

    // 1. Registry account (PDA)
    let registry_account = next_account_info(account_info_iter)?;

    // 2. Admin account (must be signer)
    let admin_account = next_account_info(account_info_iter)?;

    // 3. Mint account (game studio NFT)
    let mint_account = next_account_info(account_info_iter)?;

    // 4. Metadata account
    let metadata_account = next_account_info(account_info_iter)?;

    // 5. Metadata program
    let metadata_program = next_account_info(account_info_iter)?;

    // 6. System program
    let system_program = next_account_info(account_info_iter)?;

    // 7. Rent sysvar
    let rent_sysvar = next_account_info(account_info_iter)?;

    // **Authentication: Ensure the admin is a signer**
    if !admin_account.is_signer {
        msg!("Admin must be a signer to update game studio metadata");
        return Err(ProgramError::MissingRequiredSignature);
    }

    // **Verify that the mint account is registered in the registry**
    let mut registry_data = RegistryData::try_from_slice(&registry_account.data.borrow())
        .map_err(|_| ProgramError::InvalidAccountData)?;

    if !registry_data.game_studios.contains(mint_account.key) {
        msg!("The provided mint account is not registered as a game studio");
        return Err(ProgramError::InvalidArgument);
    }

    // **Derive the Metadata PDA**
    let (expected_metadata_pda, _) = Pubkey::find_program_address(
        &[
            b"metadata",
            &mpl_token_metadata::ID.to_bytes(),
            mint_account.key.as_ref(),
        ],
        &mpl_token_metadata::ID,
    );

    if expected_metadata_pda != *metadata_account.key {
        msg!("Invalid metadata account PDA");
        return Err(ProgramError::InvalidArgument);
    }
    // **Deserialize the existing metadata**
    let mut metadata = mpl_token_metadata::accounts::Metadata::try_from(metadata_account)
        .map_err(|_| ProgramError::InvalidAccountData)?;

    // **Update the URI if provided**
    if let Some(ref uri) = new_uri {
        metadata.uri = uri.clone();
    } else {
        msg!("No new URI provided for update");
        return Err(ProgramError::InvalidArgument);
    }

    // Create the update metadata instruction
    let update_metadata_ix = mpl_token_metadata::instructions::UpdateMetadataAccountV2 {
        metadata: *metadata_account.key,
        update_authority: *admin_account.key,
    }
    .instruction(
        mpl_token_metadata::instructions::UpdateMetadataAccountV2InstructionArgs {
            data: Some(DataV2 {
                name: metadata.name,
                symbol: metadata.symbol,
                uri: new_uri.unwrap_or(metadata.uri),
                seller_fee_basis_points: metadata.seller_fee_basis_points,
                creators: metadata.creators,
                collection: metadata.collection,
                uses: metadata.uses,
            }),
            primary_sale_happened: None,
            is_mutable: Some(true),
            new_update_authority: None,
        },
    );

    // Invoke the update instruction
    invoke(
        &update_metadata_ix,
        &[metadata_account.clone(), admin_account.clone()],
    )?;

    msg!("Game Studio metadata updated successfully");
    Ok(())
}
