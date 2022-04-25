import * as anchor from "@project-serum/anchor";
import { Provider } from "@project-serum/anchor";
import { TokenInstructions } from "@project-serum/serum";

export async function mintToAccount(
  provider: Provider,
  mint: anchor.web3.PublicKey,
  destination: anchor.web3.PublicKey,
  amount: String,
  mintAuthority: anchor.web3.PublicKey
) {
  // mint authority is the provider
  const tx = new anchor.web3.Transaction();

  tx.add(
    ...(await createMintToAccountInstrs(
      mint,
      destination,
      amount,
      mintAuthority
    ))
  );

  try{
    await provider.send(tx, []);
  } catch (err) {
    console.log(err);
  }
  return;

}

export async function createMintToAccountInstrs(
  mint: anchor.web3.PublicKey,
  destination: anchor.web3.PublicKey,
  amount: String,
  mintAuthority: anchor.web3.PublicKey
) {
  return [
    TokenInstructions.mintTo({
      mint,
      destination: destination,
      amount: amount,
      mintAuthority: mintAuthority,
    }),
  ];
}

