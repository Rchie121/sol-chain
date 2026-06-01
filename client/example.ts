// Simple frontend client example for sol-chain
// Run with: npx ts-node client/example.ts

import * as anchor from "@coral-xyz/anchor";
import { Connection, PublicKey, clusterApiUrl } from "@solana/web3.js";
import { Program, AnchorProvider, Wallet } from "@coral-xyz/anchor";

// Load IDL after `anchor build`
// import { IDL as AmmIDL } from "../target/idl/amm.json";
// import { IDL as LendingIDL } from "../target/idl/lending.json";

const connection = new Connection(clusterApiUrl("devnet"));
const wallet = Wallet.local(); // or use Phantom etc.
const provider = new AnchorProvider(connection, wallet, {});
anchor.setProvider(provider);

async function main() {
  console.log("Connected to devnet");

  // Example: Initialize AMM pool
  // const ammProgram = new Program(AmmIDL, new PublicKey("YOUR_AMM_PROGRAM_ID"), provider);
  // await ammProgram.methods.initializePool(...).rpc();

  // Example: Deposit in Lending
  // const lendingProgram = new Program(LendingIDL, new PublicKey("YOUR_LENDING_PROGRAM_ID"), provider);
  // await lendingProgram.methods.deposit(new anchor.BN(1_000_000_000)).rpc();

  console.log("Example client ready. Replace with real calls after build.");
}

main();
