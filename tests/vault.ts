import * as anchor from "@coral-xyz/anchor";
import type { Program } from "@coral-xyz/anchor";
import {
    ASSOCIATED_TOKEN_PROGRAM_ID,
  TOKEN_2022_PROGRAM_ID,
  amountToUiAmount,
  createAssociatedTokenAccount,
  createAssociatedTokenAccountInstruction,
  createInitializeMintInstruction,
  createMint,
  getAssociatedTokenAddressSync,
  getMinimumBalanceForRentExemptMint,
  getMinimumBalanceForRentExemptMintWithExtensions,
  getMint,
} from "@solana/spl-token";
import { getAccount, mintTo } from "@solana/spl-token";
import { ASSOCIATED_PROGRAM_ID } from "@coral-xyz/anchor/dist/cjs/utils/token";
import { Vault } from "../target/types/vault";
import { TOKEN_PROGRAM_ID, getAssociatedTokenAddress } from "@solana/spl-token";
import {
  PublicKey,
  sendAndConfirmTransaction,
  SystemProgram,
  Transaction,
  TransactionInstruction,
} from "@solana/web3.js";
import { BN, min } from "bn.js";
import Decimal from "decimal.js";
import { associatedTokenProgram, Metaplex } from "@metaplex-foundation/js";
import { createMintInstruction, Metadata } from "@metaplex-foundation/mpl-token-metadata";

describe("interest-bearing", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  const connection = provider.connection;
  const wallet = provider.wallet as anchor.Wallet;
  anchor.setProvider(provider);

  const program = anchor.workspace.Vault as Program<Vault>;
  it("Create Mint with InterestBearingConfig extension", async () => {
    try {
      const wallet = provider.wallet as anchor.Wallet;

      const refAddress = new PublicKey(
        "Bcch5xno7LTej6bTG55RU4FPsxW82bwUPXUm7FTDTZQV"
      );

      const hippoKeypair = anchor.web3.Keypair.generate(); // Generate a new Keypair for staking mint
      const hippoMint = await createMint(provider.connection, wallet.payer, wallet.payer.publicKey, wallet.payer.publicKey,6)
      
      // new PublicKey("AVazSc1yZY1WLXu1LUPMD5nVqJbozSDM7qibtdi3sKmk")


      console.log("hippoMint",hippoMint)

      // // 2. Create Token Accounts for the Staker
      const stakerTokenAccount = await createAssociatedTokenAccount(
        provider.connection,
        wallet.payer,
        hippoMint,
        wallet.payer.publicKey,
        null,
        TOKEN_PROGRAM_ID
        // Associated token account for the staker,
      );

      // // 3. Mint Stake Tokens to Staker Account
      const mintAmount = 10_000_000 * 10 ** 6; // 1000000 tokens with 6 decimals

      await mintTo(
        provider.connection,
        wallet.payer,
        hippoMint,
        stakerTokenAccount, // Destination account
        wallet.payer, //      Mint authority
        mintAmount,
        [],
        null,
        TOKEN_PROGRAM_ID
      );

      const seedBuffer = Buffer.from("vault");

      // Find the PDA (Program Derived Address)
      /// this is
      const [pda] = await anchor.web3.PublicKey.findProgramAddressSync(
        [seedBuffer], // Same seed as in your program
        program.programId // The program ID (same as the one used in the on-chain program)
      );

      
      console.log("wallet", wallet.publicKey)

      const connection = provider.connection;

      const metaplex = Metaplex.make(connection);

    
      await program.methods
        .init(
          new BN(5),
          wallet.payer.publicKey,
        )
        .accounts({
        })
        .signers([wallet.payer])
        .rpc({ skipPreflight: true });

        const [collectionPDA] = anchor.web3.PublicKey.findProgramAddressSync(
          [Buffer.from("Collection")],
          program.programId
        );
      
        const collectionMetadataPDA = await metaplex
        .nfts()
        .pdas()
        .metadata({ mint: collectionPDA });
  
        const collectionMasterEditionPDA = await metaplex
          .nfts()
          .pdas()
          .masterEdition({ mint: collectionPDA });
    
        const collectionTokenAccount = await getAssociatedTokenAddress(
          collectionPDA,
          wallet.publicKey
        );
    
        const modifyComputeUnits =
          anchor.web3.ComputeBudgetProgram.setComputeUnitLimit({
            units: 300_000,
          });

        const tx = await program.methods
        .createCollectionNft(
          "https://bafybeigvasvth3enh4g2ahvgic57cz2ju2vngp57bhvqidjidnycnqtxru.ipfs.w3s.link/BLACK-NINJA.json",
          "HERO",
          "HERO"
        )
        .accounts({
          authority: wallet.publicKey,
          metadataAccount: collectionMetadataPDA,
          masterEdition: collectionMasterEditionPDA,
        })
        .transaction();

        const transferTransaction = new anchor.web3.Transaction().add(
          modifyComputeUnits,
          tx
        );
    
        const txSig = await anchor.web3.sendAndConfirmTransaction(
          connection,
          transferTransaction,
          [wallet.payer],
          { skipPreflight: true }
        );


    
      await program.methods.updateStakeMint().accounts({
        stakeMint: hippoMint,
      }).rpc( { skipPreflight: true })

      // check metadata account has expected data
      const accInfo = await connection.getAccountInfo(collectionMetadataPDA);
      const metadata = Metadata.deserialize(accInfo.data, 0);
      console.log("metadata",metadata)

      for(let i=0 ; i<4 ; i ++){
        const mint = anchor.web3.Keypair.generate();
        const instructions: TransactionInstruction[] = [];
  
        // Tạo tài khoản mint mới
        instructions.push(
          SystemProgram.createAccount({
            fromPubkey: wallet.payer.publicKey,
            newAccountPubkey: mint.publicKey,
            space: 82, // Kích thước tài khoản mint (chuẩn SPL)
            lamports: await getMinimumBalanceForRentExemptMint(connection), // Rent-exempt balance
            programId: TOKEN_PROGRAM_ID,
          })
        );
  
        
      
        // Khởi tạo tài khoản mint
        instructions.push(
          createInitializeMintInstruction(
            mint.publicKey,           // PublicKey của tài khoản mint
            0,       // Số chữ số thập phân (ở đây là 0)
            collectionPDA,      // Mint authority
            collectionPDA, // Freeze authority
            TOKEN_PROGRAM_ID // ID chương trình SPL Token
          )
        )
        
        const [associatedToken] = PublicKey.findProgramAddressSync(
          [wallet.publicKey.toBuffer(), TOKEN_PROGRAM_ID.toBuffer(), mint.publicKey.toBuffer()],
          associatedTokenProgram.address
        );

        instructions.push(
          createAssociatedTokenAccountInstruction(
            wallet.payer.publicKey,
            associatedToken,
            wallet.payer.publicKey,
            mint.publicKey,
          )
        );
  
        const metadataPDA = await metaplex
        .nfts()
        .pdas()
        .metadata({ mint: mint.publicKey });

        console.log("metadataPDA",metadataPDA)
  
      const masterEditionPDA = await metaplex
        .nfts()
        .pdas()
        .masterEdition({ mint: mint.publicKey });
      
        // const destinationTokenAccount = getAssociatedTokenAddressSync(
        //   mint.publicKey,
        //   wallet.payer.publicKey,
        //   false,
        //   TOKEN_2022_PROGRAM_ID,
        //   ASSOCIATED_TOKEN_PROGRAM_ID,
        // );
  
      const boxtx = await program.methods
        .buyBox()
        .accounts({
          refWallet: wallet.publicKey,
          staker: wallet.payer.publicKey,
          stakeMint: hippoMint,
          stakerTokenAccount: stakerTokenAccount,
          collectionMetadataAccount: collectionMetadataPDA,
          collectionMasterEdition: collectionMasterEditionPDA,
          metadataAccount: metadataPDA,
          masterEdition: masterEditionPDA,
          tokenAccount:associatedToken,
          nftMint: mint.publicKey,
        })
        .instruction();
  
      const buyBoxTransferTransaction = new anchor.web3.Transaction().add(
          modifyComputeUnits,
          ...instructions,
        boxtx
      );
  
      const buyBoxTxSig = await anchor.web3.sendAndConfirmTransaction(
        connection,
        buyBoxTransferTransaction,
        [wallet.payer, mint],
        { skipPreflight: true }
      );
    }
      

    // check metadata account has expected data
    // console.log("boxMetadata",boxMetadata)
  
    const seeds = [Buffer.from('account'), wallet.publicKey.toBuffer()];

    const [userPda] = await PublicKey.findProgramAddressSync(
        seeds, // Same seed as in your program
        program.programId // The program ID (same as the one used in the on-chain program)
    )

    const user = await program.account.userInfo.fetch(userPda); // Fetch and deserialize the account data

    console.log("user",user.interestRate.toString())
    console.log("amountStake",user.amountStake.toString())

      await sleep(10)
      const reward = await  program.methods.estimateAccruredInterest().accounts({
          vaultInfo: pda,
          userInfo:userPda
      }).view()

      console.log("reward",reward.toString())

      const transaction = new Transaction();
      let refTokenAccountData;
      const refTokenAccount = await getAssociatedTokenAddress(hippoMint, refAddress,false)
      try {
          // Kiểm tra xem tài khoản token đã tồn tại hay chưa
          refTokenAccountData = await getAccount(connection, refAddress);
      } catch (error) {

          // Nếu tài khoản không tồn tại (hoặc bị lỗi do chưa được tạo), thì tạo tài khoản mới
              transaction.add(
                  createAssociatedTokenAccountInstruction(
                      wallet.payer.publicKey,
                      refTokenAccount,
                      refAddress,
                      hippoMint,
                      TOKEN_PROGRAM_ID,
                      ASSOCIATED_PROGRAM_ID
                  )
              );

      }

      transaction.add(
          await program.methods.claim().accounts({
              stakeMint:hippoMint,
              stakerTokenAccount,
              staker: wallet.publicKey,
          }).signers([wallet.payer]).instruction()
      )
      await sendAndConfirmTransaction(connection, transaction, [wallet.payer]);
      refTokenAccountData = await getAccount(connection, refTokenAccount);

      // await program.methods.transferOwner( new PublicKey('DFFnbg4bi76LkAFRigRpaiS7JR23mDQZrZr6pr9uZpJx')).accounts({
      // }).signers([wallet.payer]).rpc({skipPreflight:false})

      // await program.methods
      // .withdrawLp()
      // .accounts({
      //   stakeMint: hippoMint,
      //   signer: wallet.publicKey,
      // })
      // .signers([wallet.payer])
      // .rpc({ skipPreflight: false });

    } catch (e) {
      console.log(e);
    }
  });

  async function checkTokenBalance(connection, tokenAccountAddress) {
    try {
      const tokenAccountInfo = await getAccount(
        connection,
        tokenAccountAddress,
        null,
        TOKEN_2022_PROGRAM_ID
      );
      const tokenBalance = tokenAccountInfo.amount;
      console.log(`Số dư token sau khi mint: ${tokenBalance}`);
      return tokenBalance;
    } catch (error) {
      console.error("Lỗi khi lấy số dư token:", error);
    }
  }

  // Kiểm tra số dư token
});

function sleep(s: number) {
  return new Promise((resolve) => setTimeout(resolve, s * 1000));
}
