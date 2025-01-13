import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { MetalootRegistryProgram } from "../target/types/metaloot_registry_program";
import { assert } from "chai";
import { PublicKey } from "@solana/web3.js";
import { ASSOCIATED_TOKEN_PROGRAM_ID, createAccount, createMint, getAccount, getAssociatedTokenAddress, getMint, mintTo, TOKEN_PROGRAM_ID, transfer } from "@solana/spl-token";

describe("metaloot_registry_program", () => {
  const TOKEN_METADATA_PROGRAM_ID = new PublicKey('metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s');
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.MetalootRegistryProgram as Program<MetalootRegistryProgram>;

  // ===============================
  // 🎮 Game Studio Registry Tests
  // ===============================
  //
  // Test Suite 1: Studio Creation
  // ----------------------------
  // We're testing the fundamental ability to create a new game studio entry
  // in the MetaLoot registry. This is the cornerstone test that validates
  // the core registration process for game studios joining the ecosystem.
  //
  // Critical aspects being validated:
  // - PDA creation and storage
  // - Metadata handling
  // - Account structure integrity
  // ===============================

  it("Can create a studio with Tokens and NFT collection", async () => {
    // Generate keypairs for required accounts
    const sender = anchor.web3.Keypair.fromSecretKey(
      Uint8Array.from(
        JSON.parse(
          require('fs').readFileSync(
            require('os').homedir() + '/metaloot-keypair.json',
            'utf-8'
          )
        )
      )
    );
    const testKeypair = anchor.web3.Keypair.generate();
    const entrySeeds = anchor.web3.Keypair.generate();
    const nativeTokenKeypair = anchor.web3.Keypair.generate();
    const nftCollectionKeypair = anchor.web3.Keypair.generate();

    const entry_account = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("registry"), entrySeeds.publicKey.toBuffer()],
      program.programId
    )[0];
    // Create the studio with full metadata
    const tx = await program.methods
      .createGameStudio(
        "Test Studio",
        "TEST",
        "https://test-studio.com/metadata.json",
        sender.publicKey,
        nativeTokenKeypair.publicKey,
        nftCollectionKeypair.publicKey
      )
      .accounts({
        entrySeed: entrySeeds.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([sender])
      .rpc();

    console.log("Create studio transaction signature:", tx);

    // Fetch the created entry account and verify its data
    const entryAccount = await program.account.gameRegistryMetadata.fetch(
      anchor.web3.PublicKey.findProgramAddressSync(
        [Buffer.from("registry"), entrySeeds.publicKey.toBuffer()],
        program.programId
      )[0]
    );
    assert.equal(entryAccount.name, "Test Studio");
    assert.equal(entryAccount.symbol, "TEST");
    assert.equal(entryAccount.uri, "https://test-studio.com/metadata.json");
    assert.equal(entryAccount.authority.toBase58(), program.provider.publicKey.toBase58());
    assert.equal(entryAccount.nativeToken.toBase58(), nativeTokenKeypair.publicKey.toBase58());
    assert.equal(entryAccount.nftCollection.toBase58(), nftCollectionKeypair.publicKey.toBase58());
  });

  it("Can update a studio's metadata", async () => {
    // Generate keypairs for required accounts
    const sender = anchor.web3.Keypair.fromSecretKey(
      Uint8Array.from(
        JSON.parse(
          require('fs').readFileSync(
            require('os').homedir() + '/metaloot-keypair.json',
            'utf-8'
          )
        )
      )
    );

    const oldNativeToken = anchor.web3.Keypair.generate();
    const oldNftCollection = anchor.web3.Keypair.generate();

    const entrySeeds = anchor.web3.Keypair.generate();
    const newNativeToken = anchor.web3.Keypair.generate();
    const newNftCollection = anchor.web3.Keypair.generate();

    const entry_account = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("registry"), entrySeeds.publicKey.toBuffer()],
      program.programId
    )[0];

    // First create a studio
    await program.methods
      .createGameStudio(
        "Original Studio",
        "ORIG",
        "https://original-studio.com/metadata.json",
        sender.publicKey,
        oldNativeToken.publicKey,
        oldNftCollection.publicKey,
      )
      .accounts({
        entrySeed: entrySeeds.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([sender])
      .rpc();

    // Update the studio
    const tx = await program.methods
      .updateGameStudio(
        "Updated Studio",
        "UPDATE",
        "https://updated-studio.com/metadata.json",
        // program.provider.publicKey, -> authority
        newNativeToken.publicKey,
        newNftCollection.publicKey,
      )
      .accounts({
        entrySeed: entrySeeds.publicKey,
      })
      .signers([sender])
      .rpc();

    console.log("Update studio transaction signature:", tx);

    // Fetch and verify the updated entry account
    const entryAccount = await program.account.gameRegistryMetadata.fetch(entry_account);

    assert.equal(entryAccount.name, "Updated Studio");
    assert.equal(entryAccount.symbol, "UPDATE");
    assert.equal(entryAccount.uri, "https://updated-studio.com/metadata.json");
    assert.equal(entryAccount.authority.toBase58(), program.provider.publicKey.toBase58());
    assert.equal(entryAccount.nativeToken.toBase58(), newNativeToken.publicKey.toBase58());
    assert.equal(entryAccount.nftCollection.toBase58(), newNftCollection.publicKey.toBase58());
  });

  it("Can create and verify a player account", async () => {
    // Generate keypairs for required accounts
    const sender = anchor.web3.Keypair.fromSecretKey(
      Uint8Array.from(
        JSON.parse(
          require('fs').readFileSync(
            require('os').homedir() + '/metaloot-keypair.json',
            'utf-8'
          )
        )
      )
    );

    const entrySeed = anchor.web3.Keypair.generate();

    const playerPDA = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("player"), entrySeed.publicKey.toBuffer()],
      program.programId
    )[0];

    // Create a player account
    const tx = await program.methods
      .createPlayerAccount(
        "testPlayer123",
        "https://example.com/player/metadata.json"
      )
      .accounts({
        entrySeed: entrySeed.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([sender])
      .rpc();

    console.log("Create player account transaction signature:", tx);

    // Fetch and verify the created player account
    const playerAccount = await program.account.playerAccount.fetch(playerPDA);

    assert.equal(playerAccount.authority.toBase58(), sender.publicKey.toBase58());
    assert.equal(playerAccount.username, "testPlayer123");
    assert.ok(playerAccount.createdAt.toNumber() > 0);
    assert.equal(playerAccount.uri, "https://example.com/player/metadata.json");

    // Try to create another player account with the same seed (should fail)
    try {
      await program.methods
        .createPlayerAccount(
          "anotherPlayer",
          "https://example.com/player/another.json"
        )
        .accounts({
          playerAccount: playerPDA,
          entrySeed: entrySeed.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([sender])
        .rpc();

      assert.fail("Should not be able to create duplicate player account");
    } catch (error) {
      // console.log("failed to create duplicate player account", error);
      assert.ok(error.message.includes("Error"));
    }

    // Create another player account with different seed (should succeed)
    const newEntrySeed = anchor.web3.Keypair.generate();
    const newPlayerPDA = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("player"), newEntrySeed.publicKey.toBuffer()],
      program.programId
    )[0];

    const tx2 = await program.methods
      .createPlayerAccount(
        "differentPlayer",
        "https://example.com/player/different.json"
      )
      .accounts({
        entrySeed: newEntrySeed.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([sender])
      .rpc();

    console.log("Create second player account transaction signature:", tx2);

    // Verify the second player account
    const secondPlayerAccount = await program.account.playerAccount.fetch(newPlayerPDA);
    assert.equal(secondPlayerAccount.authority.toBase58(), sender.publicKey.toBase58());
    assert.equal(secondPlayerAccount.username, "differentPlayer");
    assert.ok(secondPlayerAccount.createdAt.toNumber() > 0);
    assert.equal(secondPlayerAccount.uri, "https://example.com/player/different.json");
  });

  it("Can update a player account's username and uri", async () => {
    // Generate keypairs for required accounts
    const sender = anchor.web3.Keypair.fromSecretKey(
      Uint8Array.from(
        JSON.parse(
          require('fs').readFileSync(
            require('os').homedir() + '/metaloot-keypair.json',
            'utf-8'
          )
        )
      )
    );
    const entrySeeds = anchor.web3.Keypair.generate();

    const player_account = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("player"), entrySeeds.publicKey.toBuffer()],
      program.programId
    )[0];

    // First create a player account
    await program.methods
      .createPlayerAccount(
        "initial_username",
        "https://example.com/player/initial.json"
      )
      .accounts({
        payer: sender.publicKey,
        playerPda: player_account,
        entrySeed: entrySeeds.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([sender])
      .rpc();

    // Update the player account
    const tx = await program.methods
      .updatePlayerAccount(
        "updated_username",
        "https://example.com/player/updated.json"
      )
      .accounts({
        payer: sender.publicKey,
        playerPda: player_account,
        entrySeed: entrySeeds.publicKey,
      })
      .signers([sender])
      .rpc();

    console.log("Update player account transaction signature:", tx);

    // Fetch and verify the updated player account
    const playerAccount = await program.account.playerAccount.fetch(
      anchor.web3.PublicKey.findProgramAddressSync(
        [Buffer.from("player"), entrySeeds.publicKey.toBuffer()],
        program.programId
      )[0]
    );

    assert.equal(playerAccount.username, "updated_username");
    assert.equal(playerAccount.uri, "https://example.com/player/updated.json");
  });

  it("Can initialize player token accounts and receive tokens", async () => {
    // Generate keypairs for required accounts
    const sender = anchor.web3.Keypair.fromSecretKey(
      Uint8Array.from(
        JSON.parse(
          require('fs').readFileSync(
            require('os').homedir() + '/metaloot-keypair.json',
            'utf-8'
          )
        )
      )
    );
    const entrySeeds = anchor.web3.Keypair.generate();
    const tokenMintKeypair = anchor.web3.Keypair.generate();

    // Find PDA for player account
    const player_pda = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("player"), entrySeeds.publicKey.toBuffer()],
      program.programId
    )[0];

    // Create token mint
    const tokenMint = await createMint(
      program.provider.connection,
      sender,
      sender.publicKey,
      null,
      9
    );

    // Create sender's ATA and mint tokens
    const senderATA = await createAccount(
      program.provider.connection,
      sender,
      tokenMint,
      sender.publicKey
    );

    // Mint 1000 tokens to sender
    await mintTo(
      program.provider.connection,
      sender,
      tokenMint,
      senderATA,
      sender.publicKey,
      1000 * 1e9
    );

    // Derive player's token ATA
    const playerTokenATA = await getAssociatedTokenAddress(
      tokenMint,
      player_pda,
      true
    );

    // Create player account first
    await program.methods
      .createPlayerAccount("testPlayer123","https://example.com/player/initial.json")
      .accounts({
        entrySeed: entrySeeds.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([sender])
      .rpc();

    // Initialize player token account
    const initTx = await program.methods
      .initializePlayerTokenAccounts()
      .accounts({
        tokenMint: tokenMint,
        playerPda: player_pda,
        playerTokenAccount: playerTokenATA,
        entrySeed: entrySeeds.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      })
      .signers([sender])
      .rpc();

    console.log("Initialize player token accounts transaction signature:", initTx);

    // Transfer tokens to player
    const transferTx = await transfer(
      program.provider.connection,
      sender,
      senderATA,
      playerTokenATA,
      sender.publicKey,
      100 * 1e9
    );

    console.log("Transfer tokens transaction signature:", transferTx);

    // Verify token balance in player's account
    const playerTokenAccount = await getAccount(
      program.provider.connection,
      playerTokenATA
    );
    assert.equal(Number(playerTokenAccount.amount), 100 * 1e9);
  });

  it("Can transfer tokens between player accounts using program function", async () => {
    const sender = anchor.web3.Keypair.fromSecretKey(
      Uint8Array.from(JSON.parse(require('fs').readFileSync(
        require('os').homedir() + '/metaloot-keypair.json', 'utf-8'
      )))
    );

    // Create two players for testing
    const player1Seeds = anchor.web3.Keypair.generate();
    const player2Seeds = anchor.web3.Keypair.generate();

    // Find PDAs for both players
    const player1_pda = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("player"), player1Seeds.publicKey.toBuffer()],
      program.programId
    )[0];
    const player2_pda = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("player"), player2Seeds.publicKey.toBuffer()],
      program.programId
    )[0];

    // Create token mint
    const tokenMint = await createMint(
      program.provider.connection,
      sender,
      sender.publicKey,
      null,
      9
    );

    // Get ATAs for both players
    const player1TokenATA = await getAssociatedTokenAddress(
      tokenMint,
      player1_pda,
      true
    );
    const player2TokenATA = await getAssociatedTokenAddress(
      tokenMint,
      player2_pda,
      true
    );

    // Create player accounts
    await program.methods
      .createPlayerAccount("player1","https://example.com/player/initial.json")
      .accounts({
        entrySeed: player1Seeds.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([sender])
      .rpc();

    await program.methods
      .createPlayerAccount("player2","https://example.com/player/initial.json")
      .accounts({
        entrySeed: player2Seeds.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([sender])
      .rpc();

    // Initialize token accounts for both players
    await program.methods
      .initializePlayerTokenAccounts()
      .accounts({
        tokenMint: tokenMint,
        playerPda: player1_pda,
        playerTokenAccount: player1TokenATA,
        entrySeed: player1Seeds.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      })
      .signers([sender])
      .rpc();

    await program.methods
      .initializePlayerTokenAccounts()
      .accounts({
        tokenMint: tokenMint,
        playerPda: player2_pda,
        playerTokenAccount: player2TokenATA,
        entrySeed: player2Seeds.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      })
      .signers([sender])
      .rpc();

    // Mint initial tokens to player1
    const senderATA = await createAccount(
      program.provider.connection,
      sender,
      tokenMint,
      sender.publicKey
    );
    await mintTo(
      program.provider.connection,
      sender,
      tokenMint,
      player1TokenATA,
      sender.publicKey,
      1000 * 1e9
    );

    // Transfer tokens using program function
    const transferAmount = 500 * 1e9;
    const transferTx = await program.methods
      .transferTokens(new anchor.BN(transferAmount))
      .accounts({
        tokenMint,
        senderSeed: player1Seeds.publicKey,
        senderPda: player1_pda,
        senderTokenAccount: player1TokenATA,
        recipientSeed: player2Seeds.publicKey,
        recipientPda: player2_pda,
        recipientTokenAccount: player2TokenATA,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([sender])
      .rpc();

    console.log("Program transfer transaction signature:", transferTx);

    // Verify token balances
    const player1TokenAccount = await getAccount(
      program.provider.connection,
      player1TokenATA
    );
    const player2TokenAccount = await getAccount(
      program.provider.connection,
      player2TokenATA
    );

    assert.equal(Number(player1TokenAccount.amount), 500 * 1e9);
    assert.equal(Number(player2TokenAccount.amount), 500 * 1e9);
  });

});
