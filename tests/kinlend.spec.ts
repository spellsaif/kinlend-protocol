
import * as anchor from "@coral-xyz/anchor";
import { BN, Program } from "@coral-xyz/anchor";
import { KinlendProtocol } from "../target/types/kinlend_protocol";
import { Keypair, LAMPORTS_PER_SOL, PublicKey, sendAndConfirmTransaction, SystemProgram } from "@solana/web3.js";
import { createMint } from "@solana/spl-token";
import NodeWallet from "@coral-xyz/anchor/dist/cjs/nodewallet";
import { SYSTEM_PROGRAM_ID } from "@coral-xyz/anchor/dist/cjs/native/system";
import assert from "assert";
import { expect } from "chai";


describe("KINLEND PROTOCOL", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  // const provider = anchor.getProvider();
  const program = anchor.workspace.KinlendProtocol as Program<KinlendProtocol>;


  //values
  let loanId;

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
  

    // ------------------------------------------------------------------
   // Config PDA
   // ------------------------------------------------------------------
  const [configPDA] = PublicKey.findProgramAddressSync(
    [Buffer.from("config")],
    program.programId
  );


   // ------------------------------------------------------------------
   // Loan Registry PDA
   // ------------------------------------------------------------------  
  
  const[loanRegistryPDA] = PublicKey.findProgramAddressSync(
    [Buffer.from("loan_registry")],
    program.programId
  );

   // ------------------------------------------------------------------
   // Loan Request PDA
   // ------------------------------------------------------------------

  loanId = 1;
  const loanIdBuffer = Buffer.alloc(8);
    loanIdBuffer.writeBigInt64LE(BigInt(loanId),0);
  
  const [loanRequestPDA] = PublicKey.findProgramAddressSync(
    [Buffer.from("loan_request"), borrower.publicKey.toBuffer(), loanIdBuffer],
    program.programId
  )

   // ------------------------------------------------------------------
   // Loan Request PDA
   // ------------------------------------------------------------------

   const [collateralVaultPDA, collateralVaultPDABump] = PublicKey.findProgramAddressSync(
    [Buffer.from("collateral_vault"), loanRequestPDA.toBuffer()],
    program.programId
   )

   const bumps = {collateralVault: collateralVaultPDABump};

   


  console.log("Config PDA:", configPDA.toBase58());


  it("Should initialize Loan Registry PDA Account", async() => {
    const latestBlockhash = await provider.connection.getLatestBlockhash();
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

    expect(loanRegistry.totalLoans.toString()).to.equal(new BN(0).toString());

    
})
  

  it("Should initialize config PDA account ", async () => {
    const latestBlockhash = await provider.connection.getLatestBlockhash();
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
    const latestBlockhash = await provider.connection.getLatestBlockhash();
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
    
    const latestBlockhash = await provider.connection.getLatestBlockhash();
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

    const latestBlockhash = await provider.connection.getLatestBlockhash();
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


  it("should create loan request PDA account and update registry", async() => {


    //Let loanAmount = 1 USDC(1_000_000 micro USDC)
    const loanAmount = new BN(1_000_000);

    //Assume Current SOL Price is 200 USD, represented as 200_000_000 mUSDC

    // required_collateral = (loan_amount * 150 * LAMPORTS_PER_SOL) / (100 * current_sol_price)
    //                           = (1_000_000 * 150 * 1e9) / (100 * 200_000_000)
    //                           = 7,500,000 lamports.
    const collateral = new BN(7_500_000);
    const noOfDays = new BN(30);


    const latestBlockhash = await provider.connection.getLatestBlockhash();

    
    const createLoanRequestTx = await program
                              .methods
                              .createLoanRequest(
                                new BN(loanId),
                                loanAmount,
                                collateral,
                                noOfDays
                              )
                              .accountsPartial({
                                borrower: borrower.publicKey,
                                loanRequest: loanRequestPDA,
                                collateralVault: collateralVaultPDA,
                                priceUpdate: priceUpdateAccount,
                                systemProgram: SYSTEM_PROGRAM_ID,
                              })
                              .signers([borrower])
                              .rpc({ commitment: "confirmed" });


      console.log("createLoanRequestTx: ", createLoanRequestTx);

      const loanRequestAccount = await program.account.loanRequestState.fetch(loanRequestPDA);

      expect(loanRequestAccount.loanId.toNumber()).to.equal(loanId);
      expect(loanRequestAccount.loanAmount.toNumber()).to.equal(loanAmount.toNumber());
      expect(loanRequestAccount.collateral.toNumber()).to.equal(collateral.toNumber());
      expect(loanRequestAccount.durationDays.toNumber()).to.equal(noOfDays.toNumber());
      expect(loanRequestAccount.borrower.toBase58()).to.equal(borrower.publicKey.toBase58());


      const collateralVaultAccount = await program.account.collateralVaultState.fetch(collateralVaultPDA);
      expect(collateralVaultAccount.bump).to.equal(collateralVaultPDABump);

      // Fetch and verify the updated loan registry.
    const loanRegistryAccount = await program.account.loanRegistryState.fetch(loanRegistryPDA);
    expect(loanRegistryAccount.loanRequests.map((pk: PublicKey) => pk.toBase58()))
      .to.include(loanRequestPDA.toBase58());
    expect(loanRegistryAccount.totalLoans.toNumber()).to.equal(1);
  })
  






});

