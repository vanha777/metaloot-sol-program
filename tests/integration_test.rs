#![cfg(test)]

use {
    borsh::{BorshDeserialize, BorshSerialize},
    solana_program_test::*,
    solana_sdk::{
        account::Account,
        instruction::{AccountMeta, Instruction},
        signature::{Keypair, Signer},
        transaction::Transaction,
        pubkey::Pubkey,
        system_instruction, 
    },
    sol_xyz::{RegistryInstruction, RegistryData, process_instruction}, // Import your program's entrypoint and structures
};
const PROGRAM_ID: Pubkey = Pubkey::from_str_const("iYwvNhYeLkb6GoUnrquy3QPyU5P3bUoWJqLSzXW4hpL");

#[tokio::test]
async fn test_initialize_and_update_registry() {
    // Step 1: Set up the test environment
    let mut test = ProgramTest::new(
        "sol_xyz",       // Symbolic name for your program
        PROGRAM_ID,         // Actual program ID
        processor!(process_instruction), // Reference to your entrypoint
    );
    test.set_compute_max_units(50_000); // Optional: increase compute budget

    // Step 2: Start the test validator and get banks client
    let (mut banks_client, payer, recent_blockhash) = test.start().await;

    // Step 3: Create a registry account keypair
    let registry_account = Keypair::new();
    let rent_exemption = banks_client
        .get_rent()
        .await
        .unwrap()
        .minimum_balance(std::mem::size_of::<RegistryData>());
    let create_registry_account_ix = system_instruction::create_account(
        &payer.pubkey(),
        &registry_account.pubkey(),
        rent_exemption,
        std::mem::size_of::<RegistryData>() as u64,
        &PROGRAM_ID,
    );

    let mut tx = Transaction::new_with_payer(
        &[create_registry_account_ix],
        Some(&payer.pubkey()),
    );
    tx.sign(&[&payer, &registry_account], recent_blockhash);
    banks_client.process_transaction(tx).await.unwrap();

    // Step 4: Initialize the registry with sample data
    let initialize_instruction = RegistryInstruction::InitializeRegistry {
        publisher_name: "Publisher".to_string(),
        publisher_logo: "https://example.com/logo.png".to_string(),
        published_date: "2024-01-01".to_string(),
        genre: "Action".to_string(),
        native_token: "TOKEN".to_string(),
        collection_symbol: "SYM".to_string(),
        collection_name: "Collection".to_string(),
        collection_description: "Description".to_string(),
        collection_image: "https://example.com/image.png".to_string(),
        time_played: "0".to_string(),
        other: None,
    };
    let initialize_instruction = borsh::to_vec(&initialize_instruction)
        .unwrap(); // Serialize the instruction

    let init_instruction = Instruction {
        program_id: PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(registry_account.pubkey(), false), // Registry account
            AccountMeta::new(payer.pubkey(), true),            // Payer (admin)
            AccountMeta::new_readonly(solana_program::system_program::id(), false), // System program
        ],
        data: initialize_instruction,
    };

    let mut tx = Transaction::new_with_payer(&[init_instruction], Some(&payer.pubkey()));
    tx.sign(&[&payer], recent_blockhash);
    banks_client.process_transaction(tx).await.unwrap();

    // Step 5: Verify the initialized data
    let registry_account_data = banks_client
        .get_account(registry_account.pubkey())
        .await
        .unwrap()
        .expect("Registry account not found");
    let registry_data: RegistryData =
        BorshDeserialize::try_from_slice(&registry_account_data.data).unwrap();

    assert!(registry_data.is_initialized);
    assert_eq!(registry_data.publisher_name, "Publisher");
    assert_eq!(registry_data.publisher_logo, "https://example.com/logo.png");

    // Step 6: Update the registry account
    let update_instruction = borsh::to_vec(&RegistryInstruction::UpdateRegistry {
        new_publisher_name: Some("Updated Publisher".to_string()),
        new_publisher_logo: None,
        new_genre: Some("Adventure".to_string()),
        new_native_token: None,
        new_collection_symbol: None,
        new_collection_name: None,
        new_collection_description: None,
        new_collection_image: None,
        new_time_played: Some("100".to_string()),
        new_other: Some("Additional Info".to_string()),
    })
    .unwrap(); // Serialize the instruction

    let update_instruction = Instruction {
        program_id: PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(registry_account.pubkey(), false), // Registry account
            AccountMeta::new(payer.pubkey(), true),            // Admin (signer)
        ],
        data: update_instruction,
    };

    let mut tx = Transaction::new_with_payer(&[update_instruction], Some(&payer.pubkey()));
    tx.sign(&[&payer], recent_blockhash);
    banks_client.process_transaction(tx).await.unwrap();

    // Step 7: Verify the updated data
    let updated_registry_account_data = banks_client
        .get_account(registry_account.pubkey())
        .await
        .unwrap()
        .expect("Registry account not found after update");
    let updated_registry_data: RegistryData =
        BorshDeserialize::try_from_slice(&updated_registry_account_data.data).unwrap();

    assert!(updated_registry_data.is_initialized);
    assert_eq!(updated_registry_data.publisher_name, "Updated Publisher");
    assert_eq!(updated_registry_data.genre, "Adventure");
    assert_eq!(updated_registry_data.time_played, "100");
    assert_eq!(updated_registry_data.other, Some("Additional Info".to_string()));

    println!("Test succeeded!");
}
