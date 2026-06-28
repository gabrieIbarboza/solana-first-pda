# Contador

My first Solana PDA deployed to devnet. This project is a tiny Anchor program that creates a PDA-backed counter, records an authority, and only lets that authority increment the number.

## What it does

- Derives one counter account as a PDA using the seed `counter`.
- Initializes the counter with `count = 0`.
- Stores the wallet that initialized the account as the authority.
- Allows only that authority to increment the counter.

The Rust program lives in [programs/contador/src/lib.rs](programs/contador/src/lib.rs), with the PDA seed defined in [programs/contador/src/constants.rs](programs/contador/src/constants.rs).

## How it works

The counter account is defined in [programs/contador/src/state/counter.rs](programs/contador/src/state/counter.rs) and contains two fields:

- `count: u64`
- `authority: Pubkey`

During `initialize`, the program:

1. Derives the PDA with the seed `counter`.
2. Creates the account with enough space for the counter state.
3. Sets `count` to `0`.
4. Saves the payer’s public key as the authority.

During `increment`, the program:

1. Re-derives the same PDA.
2. Checks that the signer matches the stored authority.
3. Increases `count` by `1`.

If someone other than the authority tries to increment the counter, the program rejects the transaction with `Unauthorized` from [programs/contador/src/error.rs](programs/contador/src/error.rs).

## Prerequisites

- Rust
- Solana CLI
- Anchor CLI
- Node.js and npm
- A Solana wallet at `~/.config/solana/id.json`

If you want to interact with devnet, make sure your wallet has devnet SOL.

## Run it locally

Build and test the program:

```bash
anchor build
anchor test
```

The tests in [programs/contador/tests/test_instructions.rs](programs/contador/tests/test_instructions.rs) verify that:

- initialization succeeds once
- initializing twice fails
- the authority can increment
- a stranger cannot increment

## Run it on devnet

This repo includes a small TypeScript script in [scripts/interact.ts](scripts/interact.ts) that connects to devnet, initializes the counter PDA, and increments it twice.

Before running it, make sure:

1. The program is deployed to devnet with the expected program id.
2. Your local Anchor IDL and generated types are up to date.
3. Your wallet file has enough devnet SOL for fees.

Then run:

```bash
ANCHOR_WALLET=~/.config/solana/id.json npx ts-node scripts/interact.ts
```

What the script does:

- connects to `devnet`
- loads your local wallet
- derives the PDA from the seed `counter`
- calls `initialize`
- fetches the counter account
- calls `increment` twice
- prints the final count and authority

## Devnet notes

- The script in [scripts/interact.ts](scripts/interact.ts) uses `anchor.web3.clusterApiUrl("devnet")`, so it will not talk to localnet unless you change that line.
- [Anchor.toml](Anchor.toml) is configured for `localnet` by default, which is useful for fast tests, while the script is aimed at devnet.
- If you redeploy the program, make sure the declared program id in [programs/contador/src/lib.rs](programs/contador/src/lib.rs) still matches the deployed address.

## Project structure

- [programs/contador/src/](programs/contador/src) - on-chain program code
- [programs/contador/tests/](programs/contador/tests) - LiteSVM-based tests
- [scripts/interact.ts](scripts/interact.ts) - devnet interaction script
- [target/idl/contador.json](target/idl/contador.json) - generated IDL

## A small milestone

This is a simple counter, but it shows a PDA flow end to end: derive the address, create the account, store authority, and enforce access control on-chain. For a first devnet deployment, that is a strong foundation to build on.