import * as anchor from '@coral-xyz/anchor'
import { Program, BN } from '@coral-xyz/anchor'
import { ComputeBudgetInstruction, ComputeBudgetProgram, Keypair, PublicKey, SystemProgram, SYSVAR_RENT_PUBKEY, Transaction } from '@solana/web3.js'
import { assert } from 'chai'
import { StateAccount, VaultsAccount } from '../../pkg/protocol'
import { Protocol } from '../../target/types/protocol'
import {
  createMint,
  createAssociatedTokenAccount,
  TOKEN_PROGRAM_ID,
  createAccount,
  mintTo,
  getAccount
} from '@solana/spl-token'
import { AdminAccounts, DotWaveAccounts, createAccounts, enableOracles, initAccounts, STATEMENT_SEED, waitFor } from '../utils/utils'

const STATE_SEED = 'state'
const provider = anchor.AnchorProvider.env()
const program = anchor.workspace.Protocol as Program<Protocol>

const minter = Keypair.generate()
const admin = Keypair.generate()
const user = Keypair.generate()
const connection = program.provider.connection
let protocolAccounts: DotWaveAccounts
let state: PublicKey
let vaults: PublicKey
let accounts: AdminAccounts
let accountBase: PublicKey
let accountQuote: PublicKey

const [statement_address, bump] = PublicKey.findProgramAddressSync(
  [Buffer.from(anchor.utils.bytes.utf8.encode(STATEMENT_SEED)), user.publicKey.toBuffer()],
  program.programId
)

anchor.setProvider(provider)


describe('Prepare vault for borrow', () => {
  before(async () => {
    const sig = await connection.requestAirdrop(admin.publicKey, 1000000000)
    await waitFor(connection, sig)

    const user_sig = await connection.requestAirdrop(user.publicKey, 1000000000)
    await waitFor(connection, user_sig)

    protocolAccounts = await initAccounts(program, admin, minter)
    const { state: g_state, vaults: g_vaults } = protocolAccounts

    accounts = {
      state: g_state,
      vaults: g_vaults,
      admin: admin.publicKey
    };

    state = g_state
    vaults = g_vaults

    await enableOracles(program, 0, accounts, admin)
  })

  it('Enables lend without open fee', async () => {
    let sig = await program.methods
      .enableLending(0, 800000, new BN(10_000_000_000), 0)
      .accounts(accounts)
      .signers([admin])
      .rpc({ skipPreflight: true })

    await waitFor(connection, sig)
  })

  it('Adds lend strategy', async () => {
    let sig = await program.methods
      .addStrategy(0, true, false, new BN(1000000), new BN(1000000))
      .accounts(accounts)
      .signers([admin])
      .rpc({ skipPreflight: true })

    await waitFor(connection, sig)
  })

  it('Overrides base oracle', async () => {
    let sig = await program.methods
      .forceOverrideOracle(0, true, 2000, 2, -3, 100)
      .accountsStrict(accounts)
      .signers([admin])
      .rpc({ skipPreflight: true })

    await waitFor(connection, sig)
  })

  it('Overrides quote oracle', async () => {
    let sig = await program.methods
      .forceOverrideOracle(0, false, 1000, 2, -3, 100)
      .accountsStrict(accounts)
      .signers([admin])
      .rpc({ skipPreflight: true })

    await waitFor(connection, sig)
  })

  it('Set lend fee', async () => {
    let sig = await program.methods
      .modifyFeeCurve(0, 1, true, new BN(1000000), new BN(0), new BN(0), new BN(100))
      .accounts(accounts)
      .signers([admin])
      .rpc({ skipPreflight: true })

    await waitFor(connection, sig)
  })

})

describe('Prepare user for borrow ', () => {


  it('Creates statement', async () => {
    let sig = await program.methods
      .createStatement()
      .accounts({
        payer: user.publicKey,
        rent: SYSVAR_RENT_PUBKEY,
        systemProgram: SystemProgram.programId,
        statement: statement_address
      })
      .signers([user])
      .rpc({ skipPreflight: true })

    await waitFor(connection, sig)
  })

  it('Creates token accounts and mints tokens to it', async () => {
    accountBase = await createAssociatedTokenAccount(
      connection,
      user,
      protocolAccounts.base,
      user.publicKey
    )

    accountQuote = await createAssociatedTokenAccount(
      connection,
      user,
      protocolAccounts.quote,
      user.publicKey
    )

    await Promise.all([mintTo(connection, user, protocolAccounts.base, accountBase, minter, 1e6), mintTo(connection, user, protocolAccounts.quote, accountQuote, minter, 1e6)])
  })

  it('deposit', async () => {
    let sig = await program.methods
      .deposit(0, 0, new BN(200000), true)
      .accountsStrict({
        state,
        vaults,
        accountBase,
        accountQuote,
        statement: statement_address,
        signer: user.publicKey,
        reserveBase: protocolAccounts.reserveBase,
        reserveQuote: protocolAccounts.reserveQuote,
        tokenProgram: TOKEN_PROGRAM_ID
      })
      .signers([user])
      .rpc({ skipPreflight: true })

    await waitFor(connection, sig)

    assert.equal((await getAccount(connection, accountBase)).amount, 800000n)
    assert.equal((await getAccount(connection, accountQuote)).amount, 600000n)
    assert.equal((await getAccount(connection, protocolAccounts.reserveBase)).amount, 200000n)
    assert.equal((await getAccount(connection, protocolAccounts.reserveQuote)).amount, 400000n)
  })
})

describe('User borrow and repays', () => {
  it('borrows 100000 token units', async () => {
    let sig = await program.methods
      .borrow(0, new BN(100000))
      .accountsStrict({
        state,
        vaults,
        accountBase,
        statement: statement_address,
        signer: user.publicKey,
        reserveBase: protocolAccounts.reserveBase,
        tokenProgram: TOKEN_PROGRAM_ID
      })
      .preInstructions([ComputeBudgetProgram.setComputeUnitLimit({
        units: 1000000
      })])
      .signers([user])
      .rpc({ skipPreflight: true })

    await waitFor(connection, sig)

    assert.equal((await getAccount(connection, accountBase)).amount, 900000n)
    assert.equal((await getAccount(connection, protocolAccounts.reserveBase)).amount, 100000n)
  })

  it('repays 100000 token units', async () => {
    let sig = await program.methods
      .repay(0, new BN(100000))
      .accountsStrict({
        state,
        vaults,
        accountBase,
        statement: statement_address,
        signer: user.publicKey,
        reserveBase: protocolAccounts.reserveBase,
        tokenProgram: TOKEN_PROGRAM_ID
      })
      .preInstructions([ComputeBudgetProgram.setComputeUnitLimit({
        units: 1000000
      })])
      .signers([user])
      .rpc({ skipPreflight: true })

    await waitFor(connection, sig)

    assert.equal((await getAccount(connection, accountBase)).amount, 800000n)
    assert.equal((await getAccount(connection, protocolAccounts.reserveBase)).amount, 200000n)

  })
})




