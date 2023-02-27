import * as anchor from '@coral-xyz/anchor'
import { BN, Program } from '@coral-xyz/anchor'
import { createAccount, createMint, getAccount, mintTo, TOKEN_PROGRAM_ID } from '@solana/spl-token'
import {
  Connection,
  Keypair,
  PublicKey,
  Signer,
  SystemProgram,
  SYSVAR_RENT_PUBKEY
} from '@solana/web3.js'
import { VaultsAccount } from '../../pkg/protocol'
import { Protocol } from '../../target/types/protocol'
import { STATE_SEED } from '../../microSdk'

export interface DotWaveAccounts {
  state: PublicKey
  vaults: PublicKey
  base: PublicKey
  quote: PublicKey
  reserveBase: PublicKey
  reserveQuote: PublicKey
}

export interface AdminAccounts {
  state: PublicKey
  vaults: PublicKey
  admin: PublicKey
}

export interface Loadable<T> {
  load(Buffer): T
}

export async function createBasicVault(
  program: Program<Protocol>,
  admin: Keypair,
  minter: Keypair
): Promise<DotWaveAccounts> {
  const result = await initAccounts(program, admin, minter)

  const accounts = {
    state: result.state,
    vaults: result.vaults,
    admin: admin.publicKey
  }

  await enableOracles(program, 0, accounts, admin)
  await program.methods
    .enableLending(0, 800000, new BN(10000_000000))
    .accounts(accounts)
    .signers([admin])
    .postInstructions([
      await program.methods
        .enableSwapping(0, 100000, new BN(10000_000000))
        .accounts(accounts)
        .signers([admin])
        .instruction(),
      await program.methods
        .addStrategy(0, true, false)
        .accounts(accounts)
        .signers([admin])
        .instruction()
    ])
    .rpc({ skipPreflight: true })

  return result
}

export async function mintTokensForUser(
  connection: Connection,
  minter: Keypair,
  user: Keypair,
  base: PublicKey,
  quote: PublicKey,
  mintAmount: number | bigint = 1e9
) {
  const [accountBase, accountQuote] = await Promise.all([
    createAccount(connection, user, base, user.publicKey),
    createAccount(connection, user, quote, user.publicKey)
  ])

  await Promise.all([
    mintTo(connection, user, base, accountBase, minter, mintAmount),
    mintTo(connection, user, quote, accountQuote, minter, mintAmount)
  ])

  return {
    accountBase,
    accountQuote
  }
}

export async function createAccounts(
  program: Program<Protocol>,
  admin: Keypair
): Promise<{ state: PublicKey; vaults: PublicKey }> {
  const vaults = Keypair.generate()

  const [state, bump] = PublicKey.findProgramAddressSync(
    [Buffer.from(anchor.utils.bytes.utf8.encode(STATE_SEED))],
    program.programId
  )

  await program.methods
    .createState()
    .accounts({
      admin: admin.publicKey,
      state,
      rent: SYSVAR_RENT_PUBKEY,
      systemProgram: SystemProgram.programId,
      vaults: vaults.publicKey
    })
    .preInstructions([
      SystemProgram.createAccount({
        fromPubkey: admin.publicKey,
        newAccountPubkey: vaults.publicKey,
        space: VaultsAccount.size(),
        lamports: await program.provider.connection.getMinimumBalanceForRentExemption(
          VaultsAccount.size()
        ),
        programId: program.programId
      })
    ])
    .signers([admin, vaults])
    .rpc({ skipPreflight: true })

  return {
    state,
    vaults: vaults.publicKey
  }
}

export async function initAccounts(
  program: Program<Protocol>,
  admin: Keypair,
  minter: Keypair
): Promise<DotWaveAccounts> {
  const vaults = Keypair.generate()
  const connection = program.provider.connection
  const [state, bump] = PublicKey.findProgramAddressSync(
    [Buffer.from(anchor.utils.bytes.utf8.encode(STATE_SEED))],
    program.programId
  )

  await program.methods
    .createState()
    .accounts({
      admin: admin.publicKey,
      state,
      rent: SYSVAR_RENT_PUBKEY,
      systemProgram: SystemProgram.programId,
      vaults: vaults.publicKey
    })
    .preInstructions([
      SystemProgram.createAccount({
        fromPubkey: admin.publicKey,
        newAccountPubkey: vaults.publicKey,
        space: VaultsAccount.size(),
        lamports: await connection.getMinimumBalanceForRentExemption(VaultsAccount.size()),
        programId: program.programId
      })
    ])
    .signers([admin, vaults])
    .rpc({ skipPreflight: true })

  const base = await createMint(connection, admin, minter.publicKey, null, 6)
  const quote = await createMint(connection, admin, minter.publicKey, null, 6)
  const reserveBase = Keypair.generate()
  const reserveQuote = Keypair.generate()

  const sig = await program.methods
    .initVault()
    .accounts({
      state,
      vaults: vaults.publicKey,
      base,
      quote,
      reserveBase: reserveBase.publicKey,
      reserveQuote: reserveQuote.publicKey,
      admin: admin.publicKey,
      tokenProgram: TOKEN_PROGRAM_ID,
      systemProgram: SystemProgram.programId
    })
    .signers([admin, reserveBase, reserveQuote])
    .rpc({ skipPreflight: true })

  return {
    state,
    vaults: vaults.publicKey,
    base,
    quote,
    reserveBase: reserveBase.publicKey,
    reserveQuote: reserveQuote.publicKey
  }
}

export async function waitFor(connection: Connection, sig: string) {
  const { blockhash, lastValidBlockHeight } = await connection.getLatestBlockhash()
  await connection.confirmTransaction(
    {
      signature: sig,
      blockhash,
      lastValidBlockHeight
    },
    'singleGossip'
  )
}

export async function sleep(ms: number) {
  return new Promise(resolve => setTimeout(resolve, ms))
}

export async function enableOracles(
  program: Program<Protocol>,
  index: number,
  accounts: AdminAccounts,
  admin: Signer
) {
  const priceFeed = Keypair.generate().publicKey
  const { state, vaults, admin: adminKey } = accounts
  await program.methods
    .enableOracle(index, 6, true, true)
    .accountsStrict({
      ...accounts,
      priceFeed
    })
    .postInstructions([
      await program.methods
        .enableOracle(index, 6, false, true)
        .accountsStrict({
          ...accounts,
          priceFeed
        })
        .instruction(),
      await program.methods
        .forceOverrideOracle(0, true, 200, 1, -2, 42)
        .accountsStrict(accounts)
        .instruction(),
      await program.methods
        .forceOverrideOracle(0, false, 1000, 2, -3, 42)
        .accountsStrict(accounts)
        .instruction()
    ])
    .signers([admin])
    .rpc({ skipPreflight: true })
}

export async function createStrategy(program, accounts, admin) {
  await program.methods
    .addStrategy(0, false, true)
    .accounts(accounts)
    .signers([admin])
    .rpc({ skipPreflight: true })
}

export async function tryFetch(connection: Connection, address: PublicKey): Promise<Buffer> {
  let data = (await connection.getAccountInfo(address))?.data
  if (!data) throw 'could not fetch account info'
  return data
}
