import * as anchor from "@project-serum/anchor";
import * as spl from '@solana/spl-token';
import { Program } from "@project-serum/anchor";
import { createMint, createTokenAccount } from "@project-serum/common";
import { ExchangeBooth } from "../target/types/exchange_booth";
import * as assert from "assert";
import { mintToAccount } from "./utilities";

describe("exchange-booth", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.Provider.env());

  const program = anchor.workspace.ExchangeBooth as Program<ExchangeBooth>;

  let anotherPerson: anchor.web3.Keypair;
  let exchangeBooth: anchor.web3.Keypair;
  let vaultA: anchor.web3.PublicKey;
  let vaultB: anchor.web3.PublicKey;
  let tokenA: anchor.web3.PublicKey;
  let tokenB: anchor.web3.PublicKey;
  let adminTokenAccount: anchor.web3.PublicKey;
  let adminTokenBAccount: anchor.web3.PublicKey;
  let vaultABump: number;
  let vaultBBump: number;

  beforeEach(async () => {
    anotherPerson = anchor.web3.Keypair.generate();
    exchangeBooth = anchor.web3.Keypair.generate();

    const signature = await program.provider.connection.requestAirdrop(program.provider.wallet.publicKey, 900000000000000);
    await program.provider.connection.confirmTransaction(signature);

    tokenA = await createMint(
      program.provider,
      program.provider.wallet.publicKey,
      18
    )

    tokenB = await createMint(
      program.provider,
      program.provider.wallet.publicKey,
      18
    )

    adminTokenAccount = await createTokenAccount(
      program.provider,
      tokenA,
      program.provider.wallet.publicKey,
    )

    adminTokenBAccount = await createTokenAccount(
      program.provider,
      tokenB,
      program.provider.wallet.publicKey,
    )

    await mintToAccount(
      program.provider,
      tokenA,
      adminTokenAccount,
      "100000",
      program.provider.wallet.publicKey
    );

    await mintToAccount(
      program.provider,
      tokenB,
      adminTokenBAccount,
      "100000",
      program.provider.wallet.publicKey
    );

    let [vaultAPubKey, bumpA] = await anchor.web3.PublicKey.findProgramAddress(
        [
          Buffer.from("vault_a"),
          exchangeBooth.publicKey.toBuffer(),
          tokenA.toBuffer(),
          program.provider.wallet.publicKey.toBuffer()
        ], 
        program.programId,
    );

    let [vaultBPubKey, bumpB] = await anchor.web3.PublicKey.findProgramAddress(
        [
          Buffer.from("vault_b"),
          exchangeBooth.publicKey.toBuffer(),
          tokenB.toBuffer(),
          program.provider.wallet.publicKey.toBuffer()
        ], 
        program.programId,
    );

    await program.rpc.initializeExchangeBooth(
      1,
      {
        accounts: {
          exchangeBooth: exchangeBooth.publicKey,
          admin: program.provider.wallet.publicKey,
          tokenA,
          tokenB,
          vaultA: vaultAPubKey,
          vaultB: vaultBPubKey,
          systemProgram: anchor.web3.SystemProgram.programId,
          tokenProgram: spl.TOKEN_PROGRAM_ID,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        },
        signers: [
          exchangeBooth
        ]
      }
    )

    vaultABump = bumpA;
    vaultBBump = bumpB;
    vaultA = vaultAPubKey,
    vaultB = vaultBPubKey
  });


  xit("Initialized Successful", async () => {
    const exchangeBoothAccount = await program.account.exchangeBooth.fetch(exchangeBooth.publicKey);

    assert.equal(vaultABump, exchangeBoothAccount.vaultABump);
    assert.equal(vaultBBump, exchangeBoothAccount.vaultBBump);
    assert.equal(1, exchangeBoothAccount.rate);
  });

  xit("Fail when initialize rate less than or equal zero", async () => {
    let exchangeBoothV2 = anchor.web3.Keypair.generate();

    let [vaultAPubKey, bumpA] = await anchor.web3.PublicKey.findProgramAddress(
        [
          Buffer.from("vault_a"),
          exchangeBoothV2.publicKey.toBuffer(),
          tokenA.toBuffer(),
          program.provider.wallet.publicKey.toBuffer()
        ], 
        program.programId,
    );

    let [vaultBPubKey, bumpB] = await anchor.web3.PublicKey.findProgramAddress(
        [
          Buffer.from("vault_b"),
          exchangeBoothV2.publicKey.toBuffer(),
          tokenB.toBuffer(),
          program.provider.wallet.publicKey.toBuffer()
        ], 
        program.programId,
    );

    try {
      await program.rpc.initializeExchangeBooth(
        0,
        {
          accounts: {
            exchangeBooth: exchangeBoothV2.publicKey,
            admin: program.provider.wallet.publicKey,
            tokenA,
            tokenB,
            vaultA: vaultAPubKey,
            vaultB: vaultBPubKey,
            systemProgram: anchor.web3.SystemProgram.programId,
            tokenProgram: spl.TOKEN_PROGRAM_ID,
            rent: anchor.web3.SYSVAR_RENT_PUBKEY,
          },
          signers: [
            exchangeBoothV2
          ]
      })
    } catch (err) {
      assert.equal(err.error.errorMessage, "Rate can not be zero!");
      return;
    }

    assert.fail('The instruction should have failed with zero rate initialization topic.');
  });

  xit("Admin allow to deposit token A to contract", async () => {
    await program.rpc.deposit(
      new anchor.BN("1000"),
      {
        accounts: {
            exchangeBooth: exchangeBooth.publicKey,
            admin: program.provider.wallet.publicKey,
            adminTokenAccount, 
            vault: vaultA,
            depositToken: tokenA,
            tokenProgram: spl.TOKEN_PROGRAM_ID,
        },
        signers: []
      }
    )

    const tokenBalance = await program.provider.connection.getTokenAccountBalance(adminTokenAccount);
    const vaultABalance = await program.provider.connection.getTokenAccountBalance(vaultA);
    assert.equal(tokenBalance.value.amount, "99000")
    assert.equal(vaultABalance.value.amount, "1000")
  })

  xit("Admin allow to deposit token B to contract", async () => {
    await program.rpc.deposit(
      new anchor.BN("1000"),
      {
        accounts: {
            exchangeBooth: exchangeBooth.publicKey,
            admin: program.provider.wallet.publicKey,
            adminTokenAccount: adminTokenBAccount, 
            vault: vaultB,
            depositToken: tokenB,
            tokenProgram: spl.TOKEN_PROGRAM_ID,
        },
        signers: []
      }
    )

    const tokenBalance = await program.provider.connection.getTokenAccountBalance(adminTokenBAccount);
    const vaultBBalance = await program.provider.connection.getTokenAccountBalance(vaultB);
    assert.equal(tokenBalance.value.amount, "99000")
    assert.equal(vaultBBalance.value.amount, "1000")
  })

  xit("Admin not allow to deposit token to wrong vault", async () => {
    try {
      await program.rpc.deposit(
        new anchor.BN("1000"),
        {
          accounts: {
              exchangeBooth: exchangeBooth.publicKey,
              admin: program.provider.wallet.publicKey,
              adminTokenAccount, 
              vault: vaultB,
              depositToken: tokenA,
              tokenProgram: spl.TOKEN_PROGRAM_ID,
          },
          signers: []
        }
      )
    } catch (err) {
      assert.equal(err.error.errorMessage, "A seeds constraint was violated");
      return;
    }

    assert.fail('The instruction should have failed with wrong vault deposit topic.');
  })

  xit("Non-admin not allow to deposit token to contract", async () => {
    try {
      await program.rpc.deposit(
        new anchor.BN("1000"),
        {
          accounts: {
              exchangeBooth: exchangeBooth.publicKey,
              admin: anotherPerson.publicKey,
              adminTokenAccount, 
              vault: vaultA,
              depositToken: tokenA,
              tokenProgram: spl.TOKEN_PROGRAM_ID,
          },
          signers: [
            anotherPerson
          ]
        }
      )
    } catch (err) {
      assert.equal(err.error.errorMessage, "You are not exchange booth admin!");
      return;
    }

    assert.fail('The instruction should have failed with admin exchange booth topic.');
  })

  xit("Admin allow to withdraw token from contract", async () => {
    const recipientTokenAccount = await createTokenAccount(
      program.provider,
      tokenA,
      program.provider.wallet.publicKey,
    );

    await program.rpc.deposit(
      new anchor.BN("1000"),
      {
        accounts: {
            exchangeBooth: exchangeBooth.publicKey,
            admin: program.provider.wallet.publicKey,
            adminTokenAccount,
            vault: vaultA,
            depositToken: tokenA,
            tokenProgram: spl.TOKEN_PROGRAM_ID,
        },
        signers: []
      }
    )

    await program.rpc.withdraw(
      new anchor.BN("500"),
      {
        accounts: {
            exchangeBooth: exchangeBooth.publicKey,
            admin: program.provider.wallet.publicKey,
            recipientTokenAccount,
            vault: vaultA,
            withdrawToken: tokenA,
            tokenProgram: spl.TOKEN_PROGRAM_ID,
        },
        signers: []
      }
    ) 
  })

  it("Admin allow to deposit token B to contract", async () => {
    await program.rpc.deposit(
      new anchor.BN("1000"),
      {
        accounts: {
            exchangeBooth: exchangeBooth.publicKey,
            admin: program.provider.wallet.publicKey,
            adminTokenAccount: adminTokenBAccount, 
            vault: vaultB,
            depositToken: tokenB,
            tokenProgram: spl.TOKEN_PROGRAM_ID,
        },
        signers: []
      }
    )

    try {

    await program.rpc.exchange(
      new anchor.BN("10"),
      {
        accounts: {
            exchangeBooth: exchangeBooth.publicKey,
            vaultA, 
            vaultB,
            tokenA,
            tokenB,
            exchanger: program.provider.wallet.publicKey,
            exchangerSendTokenAccount: adminTokenAccount,
            exchangerReceiveTokenAccount: adminTokenBAccount,
            tokenProgram: spl.TOKEN_PROGRAM_ID,
        },
        signers: []
      }
    )
    } catch (err) {
      console.log(err);
    }
  })
});
