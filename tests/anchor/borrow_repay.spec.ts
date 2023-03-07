import * as anchor from '@coral-xyz/anchor'
import { Program, BN } from '@coral-xyz/anchor'
import {
  ComputeBudgetInstruction,
  ComputeBudgetProgram,
  Keypair,
  PublicKey,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
  Transaction
} from '@solana/web3.js'
import { assert } from 'chai'
import { StateAccount, StatementAccount, VaultsAccount } from '../../pkg/protocol'
import { Protocol } from '../../target/types/protocol'
import { Oracle } from '../../target/types/oracle'
import {
  createMint,
  createAssociatedTokenAccount,
  TOKEN_PROGRAM_ID,
  createAccount,
  mintTo,
  getAccount
} from '@solana/spl-token'
import {
  AdminAccounts,
  DotWaveAccounts,
  createAccounts,
  VaultAccounts,
  enableOracles,
  initAccounts,
  waitFor,
  StateWithVaults,
  createStateWithVaults,
  addVault,
  VaultAccountsWithOracles,
  enableOracle,
  createStrategy
} from '../utils/utils'
import { STATEMENT_SEED } from '../../microSdk'

const STATE_SEED = 'state'
const provider = anchor.AnchorProvider.env()
const program = anchor.workspace.Protocol as Program<Protocol>
const oracle_program = anchor.workspace.Oracle as Program<Oracle>

const minter = Keypair.generate()
const admin = Keypair.generate()
const user = Keypair.generate()
let base_oracle: PublicKey
let quote_oracle: PublicKey
const connection = program.provider.connection

let state_with_vaults: StateWithVaults
let first_vault_accounts: VaultAccounts
let second_vault_accounts: VaultAccounts

let accounts: AdminAccounts
let accountBase: PublicKey
let accountQuote: PublicKey

const [statement_address, bump] = PublicKey.findProgramAddressSync(
  [Buffer.from(anchor.utils.bytes.utf8.encode(STATEMENT_SEED)), user.publicKey.toBuffer()],
  program.programId
)

anchor.setProvider(provider)

describe('Prepare 2 vault for borrow', () => {
  before(async () => {
    const sig = await connection.requestAirdrop(admin.publicKey, 10000000000)
    await waitFor(connection, sig)

    const user_sig = await connection.requestAirdrop(user.publicKey, 1000000000)
    await waitFor(connection, user_sig)

    const { state, vaults } = (state_with_vaults = await createStateWithVaults({ admin, program }))

    first_vault_accounts = await addVault({
      admin,
      minter: minter.publicKey,
      program,
      state,
      vaults
    })
    second_vault_accounts = await addVault({
      admin,
      minter: minter.publicKey,
      program,
      state,
      vaults
    })

  })

  it('create local oracle and enable it as base one', async () => {
    const { state, vaults } = state_with_vaults

    base_oracle = await enableOracle({
      vault: 0,
      base: true,
      decimals: 6,
      skip_init: false,
      price: new BN(200000000),
      exp: -8,
      conf: new BN(200000),
      oracle_program,
      program,
      state,
      vaults,
      admin
    })

    let data = (await connection.getAccountInfo(vaults))?.data
    assert.notEqual(data, undefined)

    if (data) {
      const vaultsAccount = VaultsAccount.load(data)
      assert.equal(vaultsAccount.vaults_len(), 2)
      assert.equal(vaultsAccount.base_oracle_enabled(0), true)
      assert.equal(
        Buffer.from(vaultsAccount.oracle_base(0)).toString('hex'),
        base_oracle.toBuffer().toString('hex')
      )
      assert.equal(vaultsAccount.get_price(0), 2000000000n)
      assert.equal(vaultsAccount.get_confidence(0), 2000000n)
    }
  })

  it('create local oracle and enable it as quote one', async () => {
    const { state, vaults } = state_with_vaults

    quote_oracle = await enableOracle({
      vault: 0,
      base: false,
      decimals: 6,
      skip_init: false,
      price: new BN(100000000),
      exp: -8,
      conf: new BN(100000),
      oracle_program,
      program,
      state,
      vaults,
      admin
    })

    let data = (await connection.getAccountInfo(vaults))?.data
    assert.notEqual(data, undefined)

    if (data) {
      const vaultsAccount = VaultsAccount.load(data)
      assert.equal(vaultsAccount.quote_oracle_enabled(0), true)
      assert.equal(
        Buffer.from(vaultsAccount.oracle_quote(0)).toString('hex'),
        quote_oracle.toBuffer().toString('hex')
      )
      assert.equal(vaultsAccount.get_price_quote(0), 1000000000n)
      assert.equal(vaultsAccount.get_confidence_quote(0), 1000000n)
    }
  })

  it('Enables lend without open fee', async () => {
    const { state, vaults } = state_with_vaults
    let timestamp = Math.floor(Date.now() / 1000);
    let sig = await program.methods
      .enableLending(0, 800000, new BN(10_000_000_000), 0, timestamp)
      .accounts({
        admin: admin.publicKey,
        ...state_with_vaults
      })
      .signers([admin])
      .rpc({ skipPreflight: true })

    await waitFor(connection, sig)
  })

  it('Adds lend strategy', async () => {
    const { state, vaults } = state_with_vaults

    await createStrategy({
      admin,
      collateral_ratio: new BN(1000000),
      liquidation_threshold: new BN(1000000),
      lend: true,
      swap: false,
      program,
      state,
      vaults,
      vault: 0
    })
  })

  it('Set lend fee', async () => {
    let sig = await program.methods
      .modifyFeeCurve(0, 1, true, new BN(1000000), new BN(0), new BN(0), new BN(100))
      .accounts({
        admin: admin.publicKey,
        ...state_with_vaults
      })
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
      first_vault_accounts.base,
      user.publicKey
    )

    accountQuote = await createAssociatedTokenAccount(
      connection,
      user,
      first_vault_accounts.quote,
      user.publicKey
    )

    await Promise.all([
      mintTo(connection, user, first_vault_accounts.base, accountBase, minter, 1e6),
      mintTo(connection, user, first_vault_accounts.quote, accountQuote, minter, 1e6)
    ])
  })

  it('deposit', async () => {
    const { state, vaults } = state_with_vaults

    let sig = await program.methods
      .deposit(0, 0, new BN(200000), true)
      .accountsStrict({
        state,
        vaults,
        accountBase,
        accountQuote,
        statement: statement_address,
        signer: user.publicKey,
        reserveBase: first_vault_accounts.reserveBase,
        reserveQuote: first_vault_accounts.reserveQuote,
        tokenProgram: TOKEN_PROGRAM_ID
      })
      .signers([user])
      .remainingAccounts([{
        isSigner: false,
        isWritable: false,
        pubkey: base_oracle
      }, {
        isSigner: false,
        isWritable: false,
        pubkey: quote_oracle
      }])
      .rpc({ skipPreflight: true })

    await waitFor(connection, sig)

    assert.equal((await getAccount(connection, accountBase)).amount, 800000n)
    assert.equal((await getAccount(connection, accountQuote)).amount, 600000n)
    assert.equal((await getAccount(connection, first_vault_accounts.reserveBase)).amount, 200000n)
    assert.equal((await getAccount(connection, first_vault_accounts.reserveQuote)).amount, 400000n)
  })
})

describe('User borrow and repays', () => {
  it('borrows 100000 token units', async () => {
    const { state, vaults } = state_with_vaults


    let sig = await program.methods
      .borrow(0, new BN(100000))
      .accountsStrict({
        state,
        vaults,
        accountBase,
        statement: statement_address,
        signer: user.publicKey,
        reserveBase: first_vault_accounts.reserveBase,
        tokenProgram: TOKEN_PROGRAM_ID
      })
      .preInstructions([
        ComputeBudgetProgram.setComputeUnitLimit({
          units: 1000000
        })
      ]).remainingAccounts([{
        isSigner: false,
        isWritable: false,
        pubkey: base_oracle
      }, {
        isSigner: false,
        isWritable: false,
        pubkey: quote_oracle
      }])
      .signers([user])
      .rpc({ skipPreflight: true })

    await waitFor(connection, sig)

    assert.equal((await getAccount(connection, accountBase)).amount, 900000n)
    assert.equal((await getAccount(connection, first_vault_accounts.reserveBase)).amount, 100000n)
  })

  it('gets borrow position info', async () => {
    const { state, vaults } = state_with_vaults

    const account_info = (await connection.getAccountInfo(statement_address))?.data
    const statement_acc = StatementAccount.load(account_info as Buffer)
    const vaults_acc_info = (await connection.getAccountInfo(vaults))?.data;
    const vaults_acc = VaultsAccount.load(vaults_acc_info as Buffer)

    const borrow_position = vaults_acc.get_borrow_position_info(0, statement_acc.buffer(), 0)

    assert.equal(borrow_position.owed_quantity, 100000n)
    assert.equal(borrow_position.borrowed_quantity, 100000n)
  })

  it('repays 100000 token units', async () => {
    const { state, vaults } = state_with_vaults

    let sig = await program.methods
      .repay(0, new BN(100000))
      .accountsStrict({
        state,
        vaults,
        accountBase,
        statement: statement_address,
        signer: user.publicKey,
        reserveBase: first_vault_accounts.reserveBase,
        tokenProgram: TOKEN_PROGRAM_ID
      })
      .preInstructions([
        ComputeBudgetProgram.setComputeUnitLimit({
          units: 1000000
        })
      ])
      .remainingAccounts([{
        isSigner: false,
        isWritable: false,
        pubkey: base_oracle
      }, {
        isSigner: false,
        isWritable: false,
        pubkey: quote_oracle
      }])
      .signers([user])
      .rpc({ skipPreflight: true })

    await waitFor(connection, sig)

    assert.equal((await getAccount(connection, accountBase)).amount, 800000n)
    assert.equal((await getAccount(connection, first_vault_accounts.reserveBase)).amount, 200000n)
  })
})
