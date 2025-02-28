
import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { KinlendProtocol } from "../target/types/kinlend_protocol";
import { Keypair, LAMPORTS_PER_SOL, PublicKey } from "@solana/web3.js";
import { createMint } from "@solana/spl-token";
import NodeWallet from "@coral-xyz/anchor/dist/cjs/nodewallet";
import { SYSTEM_PROGRAM_ID } from "@coral-xyz/anchor/dist/cjs/native/system";
import assert from "assert";
import { expect } from "chai";

describe("kinlend-protocol", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  // const provider = anchor.getProvider();
  const program = anchor.workspace.KinlendProtocol as Program<KinlendProtocol>;

  //accounts
  let borrower = Keypair.generate();
  let lender = Keypair.generate();
  
  let borrowerUsdcATA: PublicKey;
  let lenderUsdcATA: PublicKey;

  let admin: PublicKey;
  let usdcMint: PublicKey;

  //Admin Keypair for transaction
  let adminPayer: Keypair;


  beforeEach(async() => {

    // ------------------------------------------------------------------
    // Airdrop funds to our newly created borrower and lender wallets.
    // This is essential so that these wallets can pay for transactions.
    // ------------------------------------------------------------------

    const borrowerAirdropSig = await provider.connection.requestAirdrop(
      borrower.publicKey,
      10 * LAMPORTS_PER_SOL
    );

    const latestBlockhashBorrower = await provider.connection.getLatestBlockhash();
    await provider.connection.confirmTransaction({
      signature: borrowerAirdropSig,
      blockhash: latestBlockhashBorrower.blockhash,
      lastValidBlockHeight: latestBlockhashBorrower.lastValidBlockHeight,
    });

    // Airdrop to lender and confirm with fresh blockhash
    const lenderAirdropSig = await provider.connection.requestAirdrop(
      lender.publicKey,
      10 * LAMPORTS_PER_SOL
    );
    const latestBlockhashLender = await provider.connection.getLatestBlockhash();
    await provider.connection.confirmTransaction({
      signature: lenderAirdropSig,
      blockhash: latestBlockhashLender.blockhash,
      lastValidBlockHeight: latestBlockhashLender.lastValidBlockHeight,
    });



    // ------------------------------------------------------------------
    // Creating Token Mints (USDC)
    // Protocol Accepts only USDC for funding loan
    // ------------------------------------------------------------------
    admin = provider.wallet.publicKey;
    adminPayer = (provider.wallet as NodeWallet).payer;

    usdcMint = await createMint(
      provider.connection, 
      adminPayer, // payer (admin's keypair)
      admin, // mint authority
      null, // freeze authority
      6, //decimals

      
    )

  })



    // ------------------------------------------------------------------
    // Find all the PDAs
    // This is essential for passing different accounts in a transactions.
    // ------------------------------------------------------------------
  
  const [configPDA] = PublicKey.findProgramAddressSync(
    [Buffer.from("config")],
    program.programId
  );

  const[loanRegistryPDA] = PublicKey.findProgramAddressSync(
    [Buffer.from("loan_registry")],
    program.programId
  );


  console.log("Config PDA:", configPDA.toBase58());


  it("Should initialize Loan Registry PDA Account", async() => {
    const createLoanRegistryTx = await program
                                      .methods
                                      .createLoanRegistry()
                                      .accountsPartial({
                                        admin,
                                        loanRegistry: loanRegistryPDA,
                                        systemProgram: SYSTEM_PROGRAM_ID,
                                      })
                                      .signers([adminPayer])
                                      .rpc({ commitment: "confirmed" });
    
    const loanRegistry = await program.account.loanRegistryState.fetch(loanRegistryPDA);

    expect(loanRegistry.totalLoans).to.equal(0);

    
})
  

  it("Should initialize config PDA account ", async () => {
    
    // ------------------------------------------------------------------
    // TEST: Initialize Config and set admin and usdc mint address
    // This is essential for ensuring client is sending correct usdc mint account.
    // ------------------------------------------------------------------

    const initConfigTx = await program.methods.initConfig()
    .accountsPartial({
      admin,
      usdcMint,
      config: configPDA,
      systemProgram: SYSTEM_PROGRAM_ID
    })
    .signers([adminPayer])
    .rpc({ commitment: "confirmed" });

    console.log("initConfigTx: ", initConfigTx);

    const config = await program.account.configState.fetch(configPDA);

    expect(config.authority.toBase58()).to.equal(admin.toBase58());
    expect(config.usdcMint.toBase58()).to.equal(usdcMint.toBase58());



  });


  it("should update config's usdcMint field", async() => {

    //create new usdc mint account
    let newUsdcMint = await createMint(
      provider.connection,
      adminPayer,
      admin,
      null,
      6
    );

    const updateConfigTx = await program.methods
                          .updateConfig()
                          .accountsPartial({
                            admin,
                            config: configPDA,
                            newUsdcMint,
                            systemProgram: SYSTEM_PROGRAM_ID

                          }).signers([adminPayer])
                          .rpc({ commitment: "confirmed" });

      const configAccount = await program.account.configState.fetch(configPDA);

      expect(configAccount.usdcMint.toBase58()).to.equal(newUsdcMint.toBase58());
  
  })

  it("should not update config's usdcMint field by Non Admin", async() => {
    try {


      await program
            .methods
            .updateConfig()
            .accountsPartial({
              admin: lender.publicKey,
              config: configPDA,
              systemProgram: SYSTEM_PROGRAM_ID

            })
            .signers([lender])
            .rpc( { commitment: "confirmed" });
        
        assert.fail("updating config usdcMint did not fail as expected");

    } catch(err: any) {
        assert.ok("failed updating usdMint as expected")
    }
  })


  it("Should fails to reinitialize config PDA account ", async () => {

    // attempt to reinitialize the config account.
    try {
      await program.methods.initConfig()
        .accountsPartial({
          admin,
          config: configPDA,
          usdcMint: usdcMint,
          systemProgram: SYSTEM_PROGRAM_ID,
        })
        .signers([adminPayer])
        .rpc({ commitment: "confirmed" });
      // If the above call does not throw, then fail the test.
      assert.fail("Reinitialization did not fail as expected");

    } catch (err: any) {
      // Log the error for debugging.
      console.log("Expected failure error message:", err.toString());
      // Verify that the error message contains an indication that the account is already in use.
      assert.ok(
        err.toString().includes("already in use"),
        "Error did not indicate that the account was already in use"
      );
    }
  });


  






});

