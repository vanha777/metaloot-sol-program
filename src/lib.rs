
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
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program::invoke_signed,
    program_error::ProgramError,
    program_pack::IsInitialized,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction,
    sysvar::Sysvar,
};

// ----------------------------------------------------------
// 1) Instruction Enum
// ----------------------------------------------------------
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum RegistryInstruction {
    /// Initialize the registry account (and set admin).
    /// Args: (publisher_name, publisher_logo, published_date, genre, native_token,
    ///        collection_symbol, collection_name, collection_description, collection_image,
    ///        time_played, other)
    InitializeRegistry {
        publisher_name: String,
        publisher_logo: String,
        published_date: String,
        genre: String,
        native_token: String,
        collection_symbol: String,
        collection_name: String,
        collection_description: String,
        collection_image: String,
        time_played: String,
        other: Option<String>,
    },

    /// Update the registry account fields (only admin can do this).
    /// Each field is Option<String>, so you only update what's Some(...)
    UpdateRegistry {
        new_publisher_name: Option<String>,
        new_publisher_logo: Option<String>,
        new_genre: Option<String>,
        new_native_token: Option<String>,
        new_collection_symbol: Option<String>,
        new_collection_name: Option<String>,
        new_collection_description: Option<String>,
        new_collection_image: Option<String>,
        new_time_played: Option<String>,
        new_other: Option<String>,
    },
}

// ----------------------------------------------------------
// 2) On-chain Data Structure
// ----------------------------------------------------------
#[derive(BorshSerialize, BorshDeserialize, Debug, Default)]
pub struct RegistryData {
    pub is_initialized: bool,   // Flag to check if we've init'd
    pub admin: Pubkey,          // Admin who can update

    // Fields we store:
    pub publisher_name: String,
    pub publisher_logo: String,
    pub published_date: String,
    pub genre: String,
    pub native_token: String,
    pub collection_symbol: String,
    pub collection_name: String,
    pub collection_description: String,
    pub collection_image: String,
    pub time_played: String,
    pub other: Option<String>,
}

// We can add a trait for easy "is_initialized" checking
impl IsInitialized for RegistryData {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

// ----------------------------------------------------------
// 3) Entrypoint
// ----------------------------------------------------------
#[cfg(not(feature = "no-entrypoint"))]
entrypoint!(process_instruction);
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    // Deserialize the incoming instruction
    let instruction = RegistryInstruction::try_from_slice(instruction_data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    match instruction {
        RegistryInstruction::InitializeRegistry {
            publisher_name,
            publisher_logo,
            published_date,
            genre,
            native_token,
            collection_symbol,
            collection_name,
            collection_description,
            collection_image,
            time_played,
            other,
        } => {
            msg!("Instruction: InitializeRegistry");
            initialize_registry(
                program_id,
                accounts,
                publisher_name,
                publisher_logo,
                published_date,
                genre,
                native_token,
                collection_symbol,
                collection_name,
                collection_description,
                collection_image,
                time_played,
                other,
            )
        }

        RegistryInstruction::UpdateRegistry {
            new_publisher_name,
            new_publisher_logo,
            new_genre,
            new_native_token,
            new_collection_symbol,
            new_collection_name,
            new_collection_description,
            new_collection_image,
            new_time_played,
            new_other,
        } => {
            msg!("Instruction: UpdateRegistry");
            update_registry(
                program_id,
                accounts,
                new_publisher_name,
                new_publisher_logo,
                new_genre,
                new_native_token,
                new_collection_symbol,
                new_collection_name,
                new_collection_description,
                new_collection_image,
                new_time_played,
                new_other,
            )
        }
    }
}

// ----------------------------------------------------------
// 4) InitializeRegistry Processor
// ----------------------------------------------------------
#[allow(clippy::too_many_arguments)]
pub fn initialize_registry(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    publisher_name: String,
    publisher_logo: String,
    published_date: String,
    genre: String,
    native_token: String,
    collection_symbol: String,
    collection_name: String,
    collection_description: String,
    collection_image: String,
    time_played: String,
    other: Option<String>,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    // registry_account - where data will be stored
    let registry_account = next_account_info(account_info_iter)?;
    // payer_account - pays for creation, also becomes admin
    let payer_account = next_account_info(account_info_iter)?;
    // system_program
    let system_program = next_account_info(account_info_iter)?;

    // The payer must sign
    if !payer_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // If the registry_account isn't owned by our program, we create it
    // (If it's already owned, it might be re-initializing)
    if registry_account.owner != program_id {
        // Space for our RegistryData
        // Adjust this carefully based on expected max string lengths
        let space: u64 = 2048;
        let rent_exemption = Rent::get()?.minimum_balance(space as usize);

        invoke_signed(
            &system_instruction::create_account(
                payer_account.key,
                registry_account.key,
                rent_exemption,
                space,
                program_id,
            ),
            &[
                payer_account.clone(),
                registry_account.clone(),
                system_program.clone(),
            ],
            &[],
        )?;
    }

    // Deserialize existing data (or default if newly created)
    let mut registry_data: RegistryData =
        RegistryData::try_from_slice(&registry_account.data.borrow())?;

    // Check if already initialized
    if registry_data.is_initialized() {
        msg!("Registry account already initialized!");
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    // Populate fields
    registry_data.is_initialized = true;
    registry_data.admin = *payer_account.key;
    registry_data.publisher_name = publisher_name;
    registry_data.publisher_logo = publisher_logo;
    registry_data.published_date = published_date;
    registry_data.genre = genre;
    registry_data.native_token = native_token;
    registry_data.collection_symbol = collection_symbol;
    registry_data.collection_name = collection_name;
    registry_data.collection_description = collection_description;
    registry_data.collection_image = collection_image;
    registry_data.time_played = time_played;
    registry_data.other = other;

    // Save data back to account
    registry_data
        .serialize(&mut &mut registry_account.data.borrow_mut()[..])?;

    msg!("Registry initialized successfully!");
    Ok(())
}

// ----------------------------------------------------------
// 5) UpdateRegistry Processor
// ----------------------------------------------------------
#[allow(clippy::too_many_arguments)]
pub fn update_registry(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    new_publisher_name: Option<String>,
    new_publisher_logo: Option<String>,
    new_genre: Option<String>,
    new_native_token: Option<String>,
    new_collection_symbol: Option<String>,
    new_collection_name: Option<String>,
    new_collection_description: Option<String>,
    new_collection_image: Option<String>,
    new_time_played: Option<String>,
    new_other: Option<String>,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    // registry_account to update
    let registry_account = next_account_info(account_info_iter)?;
    // The admin must be the signer
    let signer_account = next_account_info(account_info_iter)?;

    // Ensure registry_account is owned by this program
    if registry_account.owner != program_id {
        msg!("Registry account not owned by this program.");
        return Err(ProgramError::IncorrectProgramId);
    }

    // Deserialize
    let mut registry_data: RegistryData =
        RegistryData::try_from_slice(&registry_account.data.borrow())?;

    if !registry_data.is_initialized() {
        msg!("Registry account is not initialized.");
        return Err(ProgramError::UninitializedAccount);
    }

    // Check admin
    if registry_data.admin != *signer_account.key {
        msg!("Signer is not the admin!");
        return Err(ProgramError::IllegalOwner);
    }
    if !signer_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Update only fields that have Some(...) values
    if let Some(name) = new_publisher_name {
        registry_data.publisher_name = name;
    }
    if let Some(logo) = new_publisher_logo {
        registry_data.publisher_logo = logo;
    }
    if let Some(g) = new_genre {
        registry_data.genre = g;
    }
    if let Some(token) = new_native_token {
        registry_data.native_token = token;
    }
    if let Some(sym) = new_collection_symbol {
        registry_data.collection_symbol = sym;
    }
    if let Some(cname) = new_collection_name {
        registry_data.collection_name = cname;
    }
    if let Some(cdesc) = new_collection_description {
        registry_data.collection_description = cdesc;
    }
    if let Some(cimage) = new_collection_image {
        registry_data.collection_image = cimage;
    }
    if let Some(tplay) = new_time_played {
        registry_data.time_played = tplay;
    }
    if let Some(oth) = new_other {
        registry_data.other = Some(oth);
    }

    // Write updated data back
    registry_data
        .serialize(&mut &mut registry_account.data.borrow_mut()[..])?;

    msg!("Registry updated successfully!");
    Ok(())
}
