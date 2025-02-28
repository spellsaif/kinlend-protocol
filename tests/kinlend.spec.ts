
import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { KinlendProtocol } from "../target/types/kinlend_protocol";
import { Keypair, LAMPORTS_PER_SOL, PublicKey } from "@solana/web3.js";
import { createMint } from "@solana/spl-token";
import NodeWallet from "@coral-xyz/anchor/dist/cjs/nodewallet";
import { SYSTEM_PROGRAM_ID } from "@coral-xyz/anchor/dist/cjs/native/system";

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

  console.log("Config PDA:", configPDA.toBase58());

  

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



  });



});


// import { describe, it } from "node:test";
// import IDL from "../target/idl/kinlend_protocol.json";
// import {KinlendProtocol} from "../target/types/kinlend_protocol";
// import { BanksClient, ProgramTestContext, startAnchor } from "solana-bankrun";
// import { Connection, LAMPORTS_PER_SOL, PublicKey } from "@solana/web3.js";
// import { BankrunProvider } from "anchor-bankrun";
// import { PythSolanaReceiver } from "@pythnetwork/pyth-solana-receiver";
// import { BankrunContextWrapper } from "../bankrun-utils/bankrun-connection";
// import { BN, Program } from "@coral-xyz/anchor";
// import { Keypair } from "@solana/web3.js";
// import { createMint } from "spl-token-bankrun";
// import { SYSTEM_PROGRAM_ID } from "@coral-xyz/anchor/dist/cjs/native/system";


// describe("Kinlend Protocol Smart Contract Test", async() => {
//     let context:ProgramTestContext;
//     let provider:BankrunProvider;
//     let bankrunContextWrapper: BankrunContextWrapper;
//     let program: Program<KinlendProtocol>;
//     let banksClient: BanksClient;
//     let signer: Keypair;

//     let lender = Keypair.generate();
//     let borrower = Keypair.generate();;



//     //PYTH NETWORK DEVNET ACCOUNT
//     const pyth = new PublicKey("UVimffxr9ow1uXYxsr4LHAcV58mLzhmwaeKvJ1pjLiE");
//     const devnetConnection = new Connection("https://api.devnet.solana.com");
//     const accountInfo = await devnetConnection.getAccountInfo(pyth);


//     context = await startAnchor(
//         "",
//         [{name:"kinlend_protocol", programId: new PublicKey(IDL.address)}],
//         [
//             {
//                 address:pyth,
//                 info: accountInfo
//             }
//         ]
//     );

//     provider = new BankrunProvider(context);

//     //FOR MORE FUNCTIONS IN BANKRUN CONNECTION
//     bankrunContextWrapper = new BankrunContextWrapper(context);

//     const connection = bankrunContextWrapper.connection.toConnection();

//     const pythSolanaReceiver = new PythSolanaReceiver({
//         connection,
//         wallet: provider.wallet

//     })

//     //SOL Price Feed ID
//     const SOL_PRICE_FEED_ID = "0xef0d8b6fda2ceba41da15d4095d1da392a0d2f8ed0c6c7bc0f4cfac8c280b56d";
//     const solUsdPriceFeedAccount = pythSolanaReceiver.getPriceFeedAccountAddress(0, SOL_PRICE_FEED_ID);

//     const feedAccountInfo = await devnetConnection.getAccountInfo(solUsdPriceFeedAccount);

//     context.setAccount(solUsdPriceFeedAccount, feedAccountInfo);

//     program = new Program<KinlendProtocol>(IDL as KinlendProtocol, provider);

//     banksClient = context.banksClient;
//     signer = provider.wallet.payer;

//    // ------------------------------------------------------------------
//   // Airdrop funds to our newly created borrower and lender wallets.
//   // This is essential so that these wallets can pay for transactions.
//   // ------------------------------------------------------------------
//   await connection.requestAirdrop(borrower.publicKey, 10 * LAMPORTS_PER_SOL);
//   await connection.requestAirdrop(lender.publicKey, 10 * LAMPORTS_PER_SOL);
//   console.log("Borrower wallet:", borrower.publicKey.toBase58());
//   console.log("Lender wallet:", lender.publicKey.toBase58());

//     const mintUsdc = await createMint(
//         banksClient,
//         signer,
//         signer.publicKey,
//         null,
//         6
//     );

//    // ------------------------------------------------------------------
//   // Example: Compute PDA for a loan request using the borrower wallet.
//   // This matches your Rust PDA seed derivation:
//   // seeds = [b"loan_request", borrower.key().as_ref(), &loan_id.to_le_bytes()]
//   // ------------------------------------------------------------------
//   const loan_id = 1;

//   //we have to convert loan_id to an 8 byte Buffer
//   const loanIdBuffer = Buffer.alloc(8);
//   loanIdBuffer.writeBigUInt64LE(BigInt(loan_id));


//     const [loanRequestPDA] = PublicKey.findProgramAddressSync(
//         [Buffer.from("loan_request"), borrower.publicKey.toBuffer(), loanIdBuffer],
//         program.programId
//     );

//     console.log("Loan Request PDA:", loanRequestPDA.toBase58());


//   // ------------------------------------------------------------------
//   // Example: Compute PDA for a collateral_vault using the loanRequest publickey.
//   // This matches your Rust PDA seed derivation:
//   // seeds = [b"collateral_vault", loan_request.key().as_ref()]
//   // ------------------------------------------------------------------
//   const [collateralVaultPDA, collateralVaultPDABump] = PublicKey.findProgramAddressSync(
//     [Buffer.from("collateral_vault"), loanRequestPDA.toBuffer()],
//     program.programId
//   );

//   console.log("Collateral Vault PDA:", collateralVaultPDA.toBase58());


//   // ------------------------------------------------------------------
//   // Example: Compute PDA for a config.
//   // This matches your Rust PDA seed derivation:
//   // seeds = [b"config"]
//   // ------------------------------------------------------------------

//   const [configPDA] = PublicKey.findProgramAddressSync(
//         [Buffer.from("config")],
//         program.programId
//     );
//     console.log("Config PDA:", configPDA.toBase58());



//   it("should create config account", async() => {
//         const createConfigTx = await program.methods.initConfig().accounts(
//             {
//                 admin: provider.wallet.publicKey,
//                 usdcMint: mintUsdc,
//             }
//         ).rpc({commitment:"confirmed"});
//   })

// //   it("should create loan request account", async() => {
    
// //         const createLoanRequestTx = await program.methods.createLoanRequest(

// //         )

// //   })

// })