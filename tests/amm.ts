import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Amm } from "../target/types/amm";
import { expect } from "chai";

describe("AMM", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.Amm as Program<Amm>;

  it("Initializes a pool", async () => {
    // TODO: Create mints, ATAs, call initialize_pool
    // expect(poolAccount.tokenAMint).to.equal(...);
  });

  it("Adds liquidity and swaps with correct math", async () => {
    // Test add_liquidity + swap using constant product
    // Verify reserves and LP tokens
  });

  it("Removes liquidity proportionally", async () => {
    // Burn LP and check token return
  });
});
