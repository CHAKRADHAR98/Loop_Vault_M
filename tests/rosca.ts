import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Rosca } from "../target/types/rosca";
import { PublicKey, Keypair, SystemProgram, SYSVAR_RENT_PUBKEY } from '@solana/web3.js';
import { TOKEN_PROGRAM_ID, createMint } from "@solana/spl-token";
import { assert } from "chai";

describe("Initialize ChitFund", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.Rosca as Program<Rosca>;
  let signer = anchor.web3.Keypair.generate();

  let mint: PublicKey;
  let fundPDA: PublicKey;
  let contributionVaultPDA: PublicKey;
  let collateralVaultPDA: PublicKey;

  before(async () => {
    // Fund signer
    const airdropSig = await provider.connection.requestAirdrop(signer.publicKey, 1000000000);
    await provider.connection.confirmTransaction(airdropSig);

    mint = await createMint(
      provider.connection,
      signer,
      provider.wallet.publicKey,
      provider.wallet.publicKey,
      6
    );

    [fundPDA] = PublicKey.findProgramAddressSync(
      [mint.toBuffer()],
      program.programId
    );

    [contributionVaultPDA] = PublicKey.findProgramAddressSync(
      [Buffer.from("contribution_vault"), mint.toBuffer()],
      program.programId
    );

    [collateralVaultPDA] = PublicKey.findProgramAddressSync(
      [Buffer.from("collateral_vault"), mint.toBuffer()],
      program.programId
    );
  });

  it("Initialize Fund", async () => {
    const tx = await program.methods
      .initChitFund(
        new anchor.BN(100_000_000),
        new anchor.BN(5),
        4,
        new anchor.BN(200_000_000),
        4,
        Array(4).fill(400_000_000).map(x => new anchor.BN(x))
      )
      .accounts({
        creator: provider.wallet.publicKey,
        contributionVault: contributionVaultPDA,
        collateralVault: collateralVaultPDA,
        chitFund: fundPDA,
        mint,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        rent: SYSVAR_RENT_PUBKEY
      })
      .rpc();

    const fund = await program.account.chitFund.fetch(fundPDA);
    assert.equal(fund.isActive, true);
    assert.equal(fund.currentCycle, 0);
  });
});