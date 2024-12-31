// tests/integration_tests.rs

use solana_program_test::*;
use solana_sdk::{
    account::Account,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use mpl_token_metadata::generated::{
    instructions::{CreateMetadataAccountV3, CreateMetadataAccountV3InstructionArgs},
    types::{Creator, DataV2},
};
use solana_program::{
    program_pack::Pack,
    sysvar::{rent::Rent, Sysvar},
};
use spl_token::state::Mint;

// Import your program's processor and instruction enums
use sol_xyz::process_instruction; // Replace `your_program` with your crate name
use sol_xyz ::{RegistryData, RegistryInstruction};

#[tokio::test]
async fn test_initialize_registry() {
    // Initialize the program test environment
    let program_id = Pubkey::new_unique();
    let mut program_test = ProgramTest::new(
        "sol_xyz", // Replace with your program's name
        program_id,
        processor!(process_instruction),
    );

    // Start the test environment
    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    // Generate a new keypair for the registry account PDA
    let registry_seeds = &[b"registry"];
    let (registry_pda, _registry_bump) = Pubkey::find_program_address(registry_seeds, &program_id);

    // Allocate space for the registry account
    program_test.add_account(
        registry_pda,
        Account {
            lamports: Rent::default().minimum_balance(RegistryData::LEN),
            data: vec![0; RegistryData::LEN],
            owner: program_id,
            executable: false,
            rent_epoch: 0,
        },
    );

    // Create the InitializeRegistry instruction
    let initialize_registry_ix = RegistryInstruction::InitializeRegistry {};

    // Serialize the instruction using Borsh
    let initialize_registry_data = initialize_registry_ix.try_to_vec().unwrap();

    // Define the accounts required for InitializeRegistry
    let initialize_registry_instruction = Instruction {
        program_id,
        accounts: vec![
            // Registry account (PDA)
            AccountMeta::new(registry_pda, false),
            // Admin account (payer)
            AccountMeta::new(payer.pubkey(), true),
            // System program
            AccountMeta::new_readonly(solana_program::system_program::id(), false),
        ],
        data: initialize_registry_data,
    };

    // Create and send the transaction
    let transaction = Transaction::new_signed_with_payer(
        &[initialize_registry_instruction],
        Some(&payer.pubkey()),
        &[&payer],
        recent_blockhash,
    );

    banks_client.process_transaction(transaction).await.unwrap();

    // Fetch and verify the registry account data
    let registry_account = banks_client
        .get_account(registry_pda)
        .await
        .expect("get_account")
        .expect("registry_account not found");

    let registry_data = RegistryData::try_from_slice(&registry_account.data).expect("deserialize RegistryData");

    assert!(registry_data.is_initialized, "Registry should be initialized");
    assert_eq!(registry_data.admin, payer.pubkey(), "Admin pubkey mismatch");
    assert!(registry_data.game_studios.is_empty(), "Game studios should be empty");
}

#[tokio::test]
async fn test_create_game_studio() {
    // Initialize the program test environment
    let program_id = Pubkey::new_unique();
    let mut program_test = ProgramTest::new(
        "your_program", // Replace with your program's name
        program_id,
        processor!(process_instruction),
    );

    // Add the MPL Token Metadata program to the test environment
    program_test.add_program("mpl_token_metadata", mpl_token_metadata::ID, None);

    // Start the test environment
    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    // Generate PDAs
    let registry_seeds = &[b"registry"];
    let (registry_pda, _registry_bump) = Pubkey::find_program_address(registry_seeds, &program_id);

    // Allocate space for the registry account
    program_test.add_account(
        registry_pda,
        Account {
            lamports: Rent::default().minimum_balance(RegistryData::LEN),
            data: vec![0; RegistryData::LEN],
            owner: program_id,
            executable: false,
            rent_epoch: 0,
        },
    );

    // Initialize the registry first
    {
        // Create the InitializeRegistry instruction
        let initialize_registry_ix = RegistryInstruction::InitializeRegistry {};

        // Serialize the instruction using Borsh
        let initialize_registry_data = initialize_registry_ix.try_to_vec().unwrap();

        // Create the InitializeRegistry instruction
        let initialize_registry_instruction = Instruction {
            program_id,
            accounts: vec![
                // Registry account (PDA)
                AccountMeta::new(registry_pda, false),
                // Admin account (payer)
                AccountMeta::new(payer.pubkey(), true),
                // System program
                AccountMeta::new_readonly(solana_program::system_program::id(), false),
            ],
            data: initialize_registry_data,
        };

        // Create and send the transaction
        let transaction = Transaction::new_signed_with_payer(
            &[initialize_registry_instruction],
            Some(&payer.pubkey()),
            &[&payer],
            recent_blockhash,
        );

        banks_client.process_transaction(transaction).await.unwrap();
    }

    // Define game studio details
    let name = "Game Studio A".to_string();
    let symbol = "GSA".to_string();
    let uri = "https://example.com/metadata_a.json".to_string();

    // Generate keypairs for Mint and Metadata accounts
    let mint = Keypair::new();
    let metadata = Keypair::new();

    // Derive Metadata PDA
    let (metadata_pda, _metadata_bump) = Pubkey::find_program_address(
        &[
            b"metadata",
            &mpl_token_metadata::ID.to_bytes(),
            mint.pubkey().as_ref(),
        ],
        &mpl_token_metadata::ID,
    );

    // Allocate space for the Mint account
    program_test.add_account(
        mint.pubkey(),
        Account {
            lamports: Rent::default().minimum_balance(Mint::LEN),
            data: vec![0; Mint::LEN],
            owner: spl_token::id(),
            executable: false,
            rent_epoch: 0,
        },
    );

    // Allocate space for the Metadata account
    program_test.add_account(
        metadata_pda,
        Account {
            lamports: Rent::default().minimum_balance(1000), // Adjust based on actual requirement
            data: vec![0; 1000], // Adjust based on actual requirement
            owner: mpl_token_metadata::ID,
            executable: false,
            rent_epoch: 0,
        },
    );

    // Create the CreateGameStudio instruction
    let create_game_studio_ix = RegistryInstruction::CreateGameStudio {
        name: name.clone(),
        symbol: symbol.clone(),
        uri: uri.clone(),
    };

    // Serialize the instruction using Borsh
    let create_game_studio_data = create_game_studio_ix.try_to_vec().unwrap();

    // Create and send the CreateGameStudio instruction
    let create_game_studio_instruction = Instruction {
        program_id,
        accounts: vec![
            // Metadata account
            AccountMeta::new(metadata_pda, false),
            // Mint account
            AccountMeta::new(mint.pubkey(), false),
            // Payer account
            AccountMeta::new(payer.pubkey(), true),
            // System program
            AccountMeta::new_readonly(solana_program::system_program::id(), false),
            // Rent sysvar
            AccountMeta::new_readonly(solana_program::sysvar::rent::id(), false),
        ],
        data: create_game_studio_data,
    };

    let transaction = Transaction::new_signed_with_payer(
        &[create_game_studio_instruction],
        Some(&payer.pubkey()),
        &[&payer],
        recent_blockhash,
    );

    banks_client.process_transaction(transaction).await.unwrap();

    // Fetch and verify the registry account data
    let registry_account = banks_client
        .get_account(registry_pda)
        .await
        .expect("get_account")
        .expect("registry_account not found");

    let registry_data = RegistryData::try_from_slice(&registry_account.data).expect("deserialize RegistryData");

    assert!(
        registry_data.game_studios.contains(&mint.pubkey()),
        "Mint should be registered in game_studios"
    );
}

#[tokio::test]
async fn test_update_game_studio() {
    // Initialize the program test environment
    let program_id = Pubkey::new_unique();
    let mut program_test = ProgramTest::new(
        "your_program", // Replace with your program's name
        program_id,
        processor!(process_instruction),
    );

    // Add the MPL Token Metadata program to the test environment
    program_test.add_program("mpl_token_metadata", mpl_token_metadata::ID, None);

    // Start the test environment
    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    // Generate PDAs
    let registry_seeds = &[b"registry"];
    let (registry_pda, _registry_bump) = Pubkey::find_program_address(registry_seeds, &program_id);

    // Allocate space for the registry account
    program_test.add_account(
        registry_pda,
        Account {
            lamports: Rent::default().minimum_balance(RegistryData::LEN),
            data: vec![0; RegistryData::LEN],
            owner: program_id,
            executable: false,
            rent_epoch: 0,
        },
    );

    // Initialize the registry first
    {
        // Create the InitializeRegistry instruction
        let initialize_registry_ix = RegistryInstruction::InitializeRegistry {};

        // Serialize the instruction using Borsh
        let initialize_registry_data = initialize_registry_ix.try_to_vec().unwrap();

        // Create the InitializeRegistry instruction
        let initialize_registry_instruction = Instruction {
            program_id,
            accounts: vec![
                // Registry account (PDA)
                AccountMeta::new(registry_pda, false),
                // Admin account (payer)
                AccountMeta::new(payer.pubkey(), true),
                // System program
                AccountMeta::new_readonly(solana_program::system_program::id(), false),
            ],
            data: initialize_registry_data,
        };

        // Create and send the transaction
        let transaction = Transaction::new_signed_with_payer(
            &[initialize_registry_instruction],
            Some(&payer.pubkey()),
            &[&payer],
            recent_blockhash,
        );

        banks_client.process_transaction(transaction).await.unwrap();
    }

    // Define game studio details
    let name = "Game Studio B".to_string();
    let symbol = "GSB".to_string();
    let uri = "https://example.com/metadata_b.json".to_string();

    // Generate keypairs for Mint and Metadata accounts
    let mint = Keypair::new();
    let metadata = Keypair::new();

    // Derive Metadata PDA
    let (metadata_pda, _metadata_bump) = Pubkey::find_program_address(
        &[
            b"metadata",
            &mpl_token_metadata::ID.to_bytes(),
            mint.pubkey().as_ref(),
        ],
        &mpl_token_metadata::ID,
    );

    // Allocate space for the Mint account
    program_test.add_account(
        mint.pubkey(),
        Account {
            lamports: Rent::default().minimum_balance(Mint::LEN),
            data: vec![0; Mint::LEN],
            owner: spl_token::id(),
            executable: false,
            rent_epoch: 0,
        },
    );

    // Allocate space for the Metadata account
    program_test.add_account(
        metadata_pda,
        Account {
            lamports: Rent::default().minimum_balance(1000), // Adjust based on actual requirement
            data: vec![0; 1000], // Adjust based on actual requirement
            owner: mpl_token_metadata::ID,
            executable: false,
            rent_epoch: 0,
        },
    );

    // Create the CreateGameStudio instruction
    let create_game_studio_ix = RegistryInstruction::CreateGameStudio {
        name: name.clone(),
        symbol: symbol.clone(),
        uri: uri.clone(),
    };

    // Serialize the instruction using Borsh
    let create_game_studio_data = create_game_studio_ix.try_to_vec().unwrap();

    // Create and send the CreateGameStudio instruction
    let create_game_studio_instruction = Instruction {
        program_id,
        accounts: vec![
            // Metadata account
            AccountMeta::new(metadata_pda, false),
            // Mint account
            AccountMeta::new(mint.pubkey(), false),
            // Payer account
            AccountMeta::new(payer.pubkey(), true),
            // System program
            AccountMeta::new_readonly(solana_program::system_program::id(), false),
            // Rent sysvar
            AccountMeta::new_readonly(solana_program::sysvar::rent::id(), false),
        ],
        data: create_game_studio_data,
    };

    let transaction = Transaction::new_signed_with_payer(
        &[create_game_studio_instruction],
        Some(&payer.pubkey()),
        &[&payer],
        recent_blockhash,
    );

    banks_client.process_transaction(transaction).await.unwrap();

    // Define new URI for updating
    let new_uri = Some("https://example.com/metadata_b_updated.json".to_string());

    // Create the UpdateGameStudio instruction
    let update_game_studio_ix = RegistryInstruction::UpdateGameStudio {
        new_uri: new_uri.clone(),
    };

    // Serialize the instruction using Borsh
    let update_game_studio_data = update_game_studio_ix.try_to_vec().unwrap();

    // Create and send the UpdateGameStudio instruction
    let update_game_studio_instruction = Instruction {
        program_id,
        accounts: vec![
            // Registry account (PDA)
            AccountMeta::new(registry_pda, false),
            // Admin account (payer)
            AccountMeta::new(payer.pubkey(), true),
            // Mint account
            AccountMeta::new(mint.pubkey(), false),
            // Metadata account
            AccountMeta::new(metadata_pda, false),
            // Metadata program
            AccountMeta::new_readonly(mpl_token_metadata::ID, false),
            // System program
            AccountMeta::new_readonly(solana_program::system_program::id(), false),
            // Rent sysvar
            AccountMeta::new_readonly(solana_program::sysvar::rent::id(), false),
        ],
        data: update_game_studio_data,
    };

    let transaction = Transaction::new_signed_with_payer(
        &[update_game_studio_instruction],
        Some(&payer.pubkey()),
        &[&payer],
        recent_blockhash,
    );

    banks_client.process_transaction(transaction).await.unwrap();

    // Fetch and verify the updated metadata
    let updated_metadata_account = banks_client
        .get_account(metadata_pda)
        .await
        .expect("get_account")
        .expect("metadata_account not found");

    // Deserialize the updated metadata using MPL Token Metadata's unpack method
    let updated_metadata = mpl_token_metadata::state::Metadata::unpack(&updated_metadata_account.data)
        .expect("unpack Metadata");

    assert_eq!(
        updated_metadata.data.uri,
        new_uri.unwrap(),
        "URI should be updated"
    );
}
