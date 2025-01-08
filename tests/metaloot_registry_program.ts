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
    const entrySeeds = anchor.web3.Keypair.generate();
    const nativeTokenKeypair = anchor.web3.Keypair.generate();
    const nftCollectionKeypair = anchor.web3.Keypair.generate();

    const entry_account = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("registry"), entrySeeds.publicKey.toBuffer()],
      program.programId
    )[0];
    // Create the studio with full metadata
    const tx = await program.methods
      .createGameStudio({
        name: "Test Studio",
        symbol: "TEST",
        uri: "https://test-studio.com/metadata.json",
        creator: program.provider.publicKey,
        nativeToken: nativeTokenKeypair.publicKey,
        nftCollection: nftCollectionKeypair.publicKey,
      })
      .accounts({
        payer: sender.publicKey,
        pda: entry_account,
        entrySeed: entrySeeds.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([])
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
    assert.equal(entryAccount.creator.toBase58(), program.provider.publicKey.toBase58());
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
      .createGameStudio({
        name: "Original Studio",
        symbol: "ORIG",
        uri: "https://original-studio.com/metadata.json",
        creator: program.provider.publicKey,
        nativeToken: oldNativeToken.publicKey,
        nftCollection: oldNftCollection.publicKey,
      })
      .accounts({
        payer: sender.publicKey,
        pda: entry_account,
        entrySeed: entrySeeds.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([])
      .rpc();

    // Update the studio
    const tx = await program.methods
      .updateGameStudio({
        name: "Updated Studio",
        symbol: "UPDATE",
        uri: "https://updated-studio.com/metadata.json",
        creator: program.provider.publicKey,
        nativeToken: newNativeToken.publicKey,
        nftCollection: newNftCollection.publicKey,
      })
      .accounts({
        payer: sender.publicKey,
        pda: entry_account,
        entrySeed: entrySeeds.publicKey,
      })
      .signers([])
      .rpc();

    console.log("Update studio transaction signature:", tx);

    // Fetch and verify the updated entry account
    const entryAccount = await program.account.gameRegistryMetadata.fetch(entry_account);

    assert.equal(entryAccount.name, "Updated Studio");
    assert.equal(entryAccount.symbol, "UPDATE");
    assert.equal(entryAccount.uri, "https://updated-studio.com/metadata.json");
    assert.equal(entryAccount.creator.toBase58(), program.provider.publicKey.toBase58());
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
      .createPlayerAccount("testPlayer123")
      .accounts({
        payer: sender.publicKey,
        playerAccount: playerPDA,
        entrySeed: entrySeed.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([])
      .rpc();

    console.log("Create player account transaction signature:", tx);

    // Fetch and verify the created player account
    const playerAccount = await program.account.playerAccount.fetch(playerPDA);

    assert.equal(playerAccount.admin.toBase58(), sender.publicKey.toBase58());
    assert.equal(playerAccount.username, "testPlayer123");
    assert.ok(playerAccount.createdAt.toNumber() > 0);

    // Try to create another player account with the same seed (should fail)
    try {
      await program.methods
        .createPlayerAccount("anotherPlayer")
        .accounts({
          payer: sender.publicKey,
          playerAccount: playerPDA,
          entrySeed: entrySeed.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([])
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
      .createPlayerAccount("differentPlayer")
      .accounts({
        payer: sender.publicKey,
        playerAccount: newPlayerPDA,
        entrySeed: newEntrySeed.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([])
      .rpc();

    console.log("Create second player account transaction signature:", tx2);

    // Verify the second player account
    const secondPlayerAccount = await program.account.playerAccount.fetch(newPlayerPDA);
    assert.equal(secondPlayerAccount.admin.toBase58(), sender.publicKey.toBase58());
    assert.equal(secondPlayerAccount.username, "differentPlayer");
    assert.ok(secondPlayerAccount.createdAt.toNumber() > 0);
  });

  it("Can update a player account's username and admin", async () => {
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
    const newAdmin = anchor.web3.Keypair.generate();

    const player_account = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("player"), entrySeeds.publicKey.toBuffer()],
      program.programId
    )[0];

    // First create a player account
    await program.methods
      .createPlayerAccount("initial_username")
      .accounts({
        payer: sender.publicKey,
        playerAccount: player_account,
        entrySeed: entrySeeds.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([])
      .rpc();

    // Update the player account
    const tx = await program.methods
      .updatePlayerAccount("updated_username", newAdmin.publicKey)
      .accounts({
        payer: sender.publicKey,
        playerAccount: player_account,
        entrySeed: entrySeeds.publicKey,
      })
      .signers([])
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
    assert.equal(playerAccount.admin.toBase58(), newAdmin.publicKey.toBase58());
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
      .createPlayerAccount("testPlayer123")
      .accounts({
        payer: sender.publicKey,
        playerAccount: player_pda,
        entrySeed: entrySeeds.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([])
      .rpc();

    // Initialize player token account
    const initTx = await program.methods
      .initializePlayerTokenAccounts()
      .accounts({
        payer: sender.publicKey,
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

  // it("Can create a fungible token for a game studio", async () => {
  //   // First create a game studio since we need it for the token
  //   const entrySeed = anchor.web3.Keypair.generate();
  //   const [registryPda] = anchor.web3.PublicKey.findProgramAddressSync(
  //     [Buffer.from("registry"), entrySeed.publicKey.toBuffer()],
  //     program.programId
  //   );
  
  //   // Create game studio first
  //   await program.methods
  //     .createGameStudio({
  //       name: "Test Studio",
  //       symbol: "TEST",
  //       uri: "https://test.uri",
  //       creator: program.provider.publicKey,
  //       nativeToken: anchor.web3.PublicKey.default,
  //       nftCollection: anchor.web3.PublicKey.default,
  //     })
  //     .accounts({
  //       payer: program.provider.publicKey,
  //       pda: registryPda,
  //       entrySeed: entrySeed.publicKey,
  //       systemProgram: anchor.web3.SystemProgram.programId,
  //     })
  //     .signers([])
  //     .rpc();
  
  //   // Now create the token
  //   const [mintToken] = anchor.web3.PublicKey.findProgramAddressSync(
  //     [Buffer.from("token"), entrySeed.publicKey.toBuffer()],
  //     program.programId
  //   );
  //   const [metadataPda] = anchor.web3.PublicKey.findProgramAddressSync(
  //     [
  //       Buffer.from("metadata"), 
  //       TOKEN_METADATA_PROGRAM_ID.toBuffer(),
  //       mintToken.toBuffer(),
  //     ],
  //     TOKEN_METADATA_PROGRAM_ID
  //   );
  
  //   await program.methods
  //     .createFungibleToken()
  //     .accounts({
  //       payer: program.provider.publicKey,
  //       pda: registryPda,
  //       mint: mintToken,
  //       metadata: metadataPda,
  //       tokenMetadataProgram: TOKEN_METADATA_PROGRAM_ID,
  //       entrySeed: entrySeed.publicKey,
  //       systemProgram: anchor.web3.SystemProgram.programId,
  //       tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
  //       rent: anchor.web3.SYSVAR_RENT_PUBKEY,
  //     })
  //     .rpc();
  
  //   // Verify the token was created
  //   const mintInfo = await getMint(program.provider.connection, mintToken);
  //   assert.ok(mintInfo.mintAuthority.equals(registryPda));
  //   assert.ok(mintInfo.freezeAuthority.equals(registryPda));
  //   assert.equal(mintInfo.decimals, 9);
  
  //   // Verify the metadata was created
  //   const metadataAccount = await program.provider.connection.getAccountInfo(metadataPda);
  //   assert.ok(metadataAccount !== null);
  // });

});
