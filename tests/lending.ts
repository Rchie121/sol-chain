import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Lending } from "../target/types/lending";
import { expect } from "chai";

describe("Lending Protocol", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.Lending as Program<Lending>;

  it("Initializes market and deposits", async () => {
    // Call initialize_market + deposit
  });

  it("Allows borrowing within LTV and repays", async () => {
    // borrow() then repay()
  });

  it("Prevents over-borrowing and allows liquidation when unhealthy", async () => {
    // Test liquidation path
  });
});
