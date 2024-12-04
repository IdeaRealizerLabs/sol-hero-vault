import { createAssociatedTokenAccount, createAssociatedTokenAccountInstruction, createMint, getAssociatedTokenAddress, getAssociatedTokenAddressSync, getOrCreateAssociatedTokenAccount, mintTo, TOKEN_2022_PROGRAM_ID, TOKEN_PROGRAM_ID } from "@solana/spl-token";
import * as anchor from '@coral-xyz/anchor';
import { ASSOCIATED_PROGRAM_ID } from "@coral-xyz/anchor/dist/cjs/utils/token";
import * as fs from 'fs';
import { tokenProgram } from "@metaplex-foundation/js";
import { SYSVAR_RENT_PUBKEY } from "@solana/web3.js";
import { SYSTEM_PROGRAM_ID } from "@coral-xyz/anchor/dist/cjs/native/system";
import { Vault } from "../target/types/vault";
import * as web3 from "@solana/web3.js";
import { createUmi } from '@metaplex-foundation/umi-bundle-defaults';
import { fromWeb3JsKeypair, fromWeb3JsPublicKey } from '@metaplex-foundation/umi-web3js-adapters';
import {
  createSignerFromKeypair,
  generateSigner,
  percentAmount,
  publicKey,
  PublicKey,
  signerIdentity,
} from '@metaplex-foundation/umi'
import {
  createCreateMetadataAccountV3Instruction,
  TokenStandard,
} from '@metaplex-foundation/mpl-token-metadata'
import path from 'path'
async function main() {
  const filePath = "/Users/dungbui/Documents/id.json";
  const secretKeyString = fs.readFileSync(path.resolve(filePath), 'utf-8');

  // Chuyển đổi từ chuỗi JSON sang mảng số và tạo Keypair
  const secretKey = Uint8Array.from(JSON.parse(secretKeyString));
  const keypair = web3.Keypair.fromSecretKey(secretKey);

  const seedBuffer = Buffer.from('authority_seed');

  // Khởi tạo provider
  const connection = new web3.Connection('https://api.devnet.solana.com', 'confirmed');

  const mUSDMint ={
    publicKey: new web3.PublicKey("Eyo515hVUsV4ZMTNmQHZNtuTXhZaHCeqnRpUEgMryRHY")
  }
  const smUSCMintKeypair = new anchor.web3.Keypair();

  console.log("smUSCMintKeypair",smUSCMintKeypair.publicKey.toString())

  try {

    const program = anchor.workspace.Vault as anchor.Program<Vault>;
    await program.methods.initialize(800, 'smUSD', 'smUSD', '').accounts(
      {
          initializer: keypair.publicKey,
          stakeMint: mUSDMint.publicKey,
          rewardMint: smUSCMintKeypair.publicKey,
      }
  ).signers([smUSCMintKeypair]).rpc({ skipPreflight: false })

    const [pda, bump] = await anchor.web3.PublicKey.findProgramAddressSync(
      [seedBuffer], // Same seed as in your program
      program.programId     // The program ID (same as the one used in the on-chain program)
  );
   const vaultTokenAccount = await getAssociatedTokenAddress(mUSDMint.publicKey, pda, true,
                TOKEN_PROGRAM_ID, ASSOCIATED_PROGRAM_ID
            )
  console.log("vaultTokenAccount",vaultTokenAccount.toString())

  

  } catch (error) {
    console.error('Error initializing mint:', error);
  }
};

// Use this in your deployment script file (e.g., deploy.js) and call it with the provider
main().catch(err => {
  console.error(err);
});