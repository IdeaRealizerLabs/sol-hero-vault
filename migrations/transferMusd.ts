import { getAssociatedTokenAddress, mintTo, TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { ASSOCIATED_PROGRAM_ID } from "@coral-xyz/anchor/dist/cjs/utils/token";
import * as fs from 'fs';
import * as web3 from "@solana/web3.js";
import path from 'path';

async function main() {
  console.log("chay vao day")
  // Configure client to use the provider.
//   const provider = anchor.AnchorProvider.env();
//   anchor.setProvider(provider);

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
  
  try {
const programId = "9JphibYohaNSpdmpujRpuGr9P6MNwBAmPEh7fuJgTLtd";

  const [pda, bump] = await web3.PublicKey.findProgramAddressSync(
        [seedBuffer], // Same seed as in your program
        new web3.PublicKey(programId)     // The program ID (same as the one used in the on-chain program)
    );

   const vaultTokenAccount = await getAssociatedTokenAddress(mUSDMint.publicKey, pda, true,
                TOKEN_PROGRAM_ID, ASSOCIATED_PROGRAM_ID
            )
    const mintRewardAmount = 10 * 10 ** 6; // 1000 tokens with 6 decimals
    await mintTo(
        connection,
        keypair,
        mUSDMint.publicKey,
        vaultTokenAccount, // Destination account
        keypair, //      Mint authority
        mintRewardAmount,
        [],
        null,
        TOKEN_PROGRAM_ID
    );
  

  } catch (error) {
    console.error('Error initializing mint:', error);
  }
};

// Use this in your deployment script file (e.g., deploy.js) and call it with the provider
main().catch(err => {
  console.error(err);
});