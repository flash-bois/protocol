import * as anchor from '@coral-xyz/anchor'
import { Program } from '@coral-xyz/anchor'
import {
  Keypair,
  PublicKey,
  SIGNATURE_LENGTH_IN_BYTES,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
  Transaction
} from '@solana/web3.js'
import { assert, use } from 'chai'
import {
  price_denominator,
  StateAccount,
  StatementAccount,
  VaultsAccount
} from '../../pkg/protocol'
import { Protocol } from '../../target/types/protocol'
import {
  createMint,
  createAssociatedTokenAccount,
  TOKEN_PROGRAM_ID,
  createAccount,
  mintTo,
  getAssociatedTokenAddressSync,
  getAccount,
  getOrCreateAssociatedTokenAccount
} from '@solana/spl-token'
import {
  createAccounts,
  initAccounts,
  sleep,
  waitFor,
  enableOracles,
  AdminAccounts,
  tryFetch,
  createBasicVault,
  mintTokensForUser,
  DotWaveAccounts,
  TestEnvironment,
  createTestEnvironment,
  IVaultAccounts
} from '../utils/utils'
import { BN } from 'bn.js'
import { STATEMENT_SEED, STATE_SEED } from '../../microSdk'
import { Oracle } from '../../target/types/oracle'

const provider = anchor.AnchorProvider.env()
const program = anchor.workspace.Protocol as Program<Protocol>
const oracle_program = anchor.workspace.Oracle as Program<Oracle>

const minter = Keypair.generate()
const admin = Keypair.generate()
const user = Keypair.generate()
const connection = program.provider.connection
anchor.setProvider(provider)

let vaults_account: VaultsAccount
let statement_acc: StatementAccount
let accountBase: PublicKey
let accountQuote: PublicKey
let test_environment: TestEnvironment
let vault0: IVaultAccounts
let vault1: IVaultAccounts
let quote_mint: PublicKey

const [statement, bump] = PublicKey.findProgramAddressSync(
  [Buffer.from(anchor.utils.bytes.utf8.encode(STATEMENT_SEED)), user.publicKey.toBuffer()],
  program.programId
)

describe('User', () => {
  before(async function () {
    const admin_sig = await connection.requestAirdrop(admin.publicKey, 10000000000)
    await waitFor(connection, admin_sig)

    const user_sig = await connection.requestAirdrop(user.publicKey, 1000000000)
    await waitFor(connection, user_sig)

    quote_mint = await createMint(connection, admin, minter.publicKey, null, 6)

    test_environment = await createTestEnvironment({
      ix_only: false,
      admin,
      minter: minter.publicKey,
      oracle_program,
      program,
      vaults_infos: [
        {
          quote_mint,
          base_oracle: {
            base: true,
            decimals: 6,
            skip_init: false,
            price: new BN(200000000),
            exp: -8,
            conf: new BN(200000),
            max_update_interval: 100
          },
          quote_oracle: {
            base: false,
            decimals: 6,
            skip_init: false,
            price: new BN(100000000),
            exp: -8,
            conf: new BN(100000),
            max_update_interval: 100
          },
          lending: {
            initial_fee_time: 0,
            max_borrow: new BN(10_000_000_000),
            max_utilization: 800000
          },
          swapping: {
            kept_fee: 100000,
            max_total_sold: new BN(10_000_000_000)
          },
          strategies: [
            {
              collateral_ratio: new BN(1000000),
              liquidation_threshold: new BN(1000000),
              lend: true,
              swap: true,
              trade: false
            }
          ]
        },
        {
          quote_mint,
          base_oracle: {
            base: true,
            decimals: 6,
            skip_init: false,
            price: new BN(200000000),
            exp: -8,
            conf: new BN(200000),
            max_update_interval: 100
          },
          quote_oracle: {
            base: false,
            decimals: 6,
            skip_init: false,
            price: new BN(100000000),
            exp: -8,
            conf: new BN(100000),
            max_update_interval: 100
          },
          lending: {
            initial_fee_time: 0,
            max_borrow: new BN(10_000_000_000),
            max_utilization: 800000
          },
          swapping: {
            kept_fee: 100000,
            max_total_sold: new BN(10_000_000_000)
          },
          strategies: [
            {
              collateral_ratio: new BN(1000000),
              liquidation_threshold: new BN(1000000),
              lend: false,
              swap: true,
              trade: false
            }
          ]
        }
      ]
    })

    vault0 = test_environment.vaults_data[0]
    vault1 = test_environment.vaults_data[1]

    accountBase = await createAssociatedTokenAccount(connection, user, vault0.base, user.publicKey)

    accountQuote = await createAssociatedTokenAccount(
      connection,
      user,
      vault0.quote,
      user.publicKey
    )

    await Promise.all([
      mintTo(connection, user, vault0.base, accountBase, minter, 1e6),
      mintTo(connection, user, vault0.quote, accountQuote, minter, 1e6)
    ])
  })

  it('create statement', async () => {
    await program.methods
      .createStatement()
      .accountsStrict({
        statement,
        payer: user.publicKey,
        systemProgram: SystemProgram.programId,
        rent: SYSVAR_RENT_PUBKEY
      })
      .signers([user])
      .rpc({ skipPreflight: true })
  })

  it('calculates deposit', async () => {
    const data = (await connection.getAccountInfo(test_environment.vaults))?.data
    vaults_account = VaultsAccount.load(data as Buffer)

    assert.equal(vaults_account.deposit(0, 0, 200000n, true, 0), 400000n)
    assert.equal(vaults_account.deposit(0, 0, 400000n, false, 0), 200000n)
  })

  it('deposit', async () => {
    const remaining_accounts = vault0.remaining_accounts

    assert.equal((await getAccount(connection, accountBase)).amount, 1000000n)
    assert.equal((await getAccount(connection, accountQuote)).amount, 1000000n)

    const sig = await program.methods
      .deposit(0, 0, new BN(200000), true)
      .accountsStrict({
        ...test_environment,
        accountBase,
        accountQuote,
        statement,
        signer: user.publicKey,
        reserveBase: vault0.reserveBase,
        reserveQuote: vault0.reserveQuote,
        tokenProgram: TOKEN_PROGRAM_ID
      })
      .signers([user])
      .remainingAccounts(remaining_accounts ?? [])
      .rpc({ skipPreflight: true })

    await waitFor(connection, sig)

    assert.equal((await getAccount(connection, accountBase)).amount, 800000n)
    assert.equal((await getAccount(connection, accountQuote)).amount, 600000n)
  })

  it('gets lp position info from statement', async () => {
    const data = (await connection.getAccountInfo(statement))?.data
    statement_acc = StatementAccount.load(data as Buffer)
    const vaults_data = (await connection.getAccountInfo(test_environment.vaults))?.data
    vaults_account.reload(vaults_data as Buffer)

    const position_info = vaults_account.get_lp_position_info(0, 0, statement_acc.buffer(), 0)
    const withdraw_info = vaults_account.withdraw(0, 0, 200000n, true, statement_acc.buffer())!

    assert.equal(withdraw_info.base, 200000n)
    assert.equal(withdraw_info.quote, 400000n)

    const reverse_withdraw_info = vaults_account.withdraw(0, 0, 400000n, false, statement_acc.buffer())

    assert.equal(reverse_withdraw_info.base, 200000n)
    assert.equal(reverse_withdraw_info.quote, 400000n)

    const withdraw_half = vaults_account.withdraw(0, 0, 100000n, true, statement_acc.buffer())

    assert.equal(withdraw_half.base, 100000n)
    assert.equal(withdraw_half.quote, 200000n)

    const withdraw_half_reverse = vaults_account.withdraw(0, 0, 200000n, false, statement_acc.buffer())

    assert.equal(withdraw_half_reverse.base, 100000n)
    assert.equal(withdraw_half_reverse.quote, 200000n)

    const too_much_withdraw_info = vaults_account.withdraw(0, 0, 500000n, true, statement_acc.buffer())

    assert.equal(too_much_withdraw_info.base, 200000n)
    assert.equal(too_much_withdraw_info.quote, 400000n)

    const too_much_withdraw_info_reverse = vaults_account.withdraw(0, 0, 500000n, false, statement_acc.buffer())

    assert.equal(too_much_withdraw_info.base, 200000n)
    assert.equal(too_much_withdraw_info.quote, 400000n)



    assert.equal(position_info!.earned_base_quantity, 200000n)
    assert.equal(position_info!.earned_quote_quantity, 400000n)
    assert.equal(position_info!.position_value, 800000000n)
    assert.equal(position_info!.max_withdraw_base, 200000n)
    assert.equal(position_info!.max_withdraw_quote, 400000n)
  })

  it('single swap', async () => {
    const remaining_accounts = vault0.remaining_accounts

    assert.equal((await getAccount(connection, accountBase)).amount, 800000n)
    assert.equal((await getAccount(connection, accountQuote)).amount, 600000n)

    await program.methods
      .modifyFeeCurve(0, 2, true, new BN(1000000), new BN(0), new BN(0), new BN(10000))
      .accounts({ admin: admin.publicKey, ...test_environment })
      .signers([admin])
      .rpc({ skipPreflight: true })

    await program.methods
      .singleSwap(0, new BN(100000), new BN(10), true, false)
      .accountsStrict({
        ...test_environment,
        accountBase,
        accountQuote,
        signer: user.publicKey,
        reserveBase: vault0.reserveBase,
        reserveQuote: vault0.reserveQuote,
        tokenProgram: TOKEN_PROGRAM_ID
      })
      .remainingAccounts(remaining_accounts ?? [])
      .signers([user])
      .rpc({ skipPreflight: true })

    assert.equal((await getAccount(connection, accountBase)).amount, 700000n)
    assert.equal((await getAccount(connection, accountQuote)).amount, 800000n - 2000n)
  })

  it('double swap', async () => {
    const remaining_accounts0 = vault0.remaining_accounts
    const remaining_accounts1 = vault1.remaining_accounts
    const remaining_accounts = [...remaining_accounts0!, ...remaining_accounts1!]

    assert.equal(vault1.quote.toBase58(), vault0.quote.toBase58())
    assert.notEqual(vault1.base.toBase58(), vault0.base.toBase58())

    const accountInfo = await connection.getAccountInfo(test_environment.vaults)
    assert.isNotNull(accountInfo)
    const vaultsAccount = VaultsAccount.load(accountInfo!.data)
    assert.equal(
      new PublicKey(vaultsAccount.base_reserve(1)).toString(),
      vault1.reserveBase.toString()
    )

    const secondAccountBase = await createAssociatedTokenAccount(
      connection,
      user,
      vault1.base,
      user.publicKey
    )

    await mintTo(connection, user, vault1.base, secondAccountBase, minter, 1e6)

    const vault1_remaining_accounts = vault1.remaining_accounts
    await program.methods
      .deposit(1, 0, new BN(300000), true)
      .accountsStrict({
        ...test_environment,
        accountBase: secondAccountBase,
        accountQuote,
        statement,
        signer: user.publicKey,
        reserveBase: vault1.reserveBase,
        reserveQuote: vault1.reserveQuote,
        tokenProgram: TOKEN_PROGRAM_ID
      })
      .remainingAccounts(vault1_remaining_accounts ?? [])
      .remainingAccounts(remaining_accounts)
      .signers([user])
      .rpc({ skipPreflight: true })

    await program.methods
      .modifyFeeCurve(1, 2, false, new BN(1000000), new BN(0), new BN(0), new BN(10000))
      .accounts({
        admin: admin.publicKey,
        ...test_environment
      })
      .signers([admin])
      .rpc({ skipPreflight: true })

    assert.equal((await getAccount(connection, accountBase)).amount, 700000n)
    assert.equal((await getAccount(connection, secondAccountBase)).amount, 700000n)

    await program.methods
      .doubleSwap(0, 1, new BN(100000), new BN(10), false)
      .accountsStrict({
        ...test_environment,
        accountIn: accountBase,
        accountOut: secondAccountBase,
        signer: user.publicKey,
        reserveIn: vault0.reserveBase,
        reserveInQuote: vault0.reserveQuote,
        reserveOut: vault1.reserveBase,
        reserveOutQuote: vault1.reserveQuote,
        tokenProgram: TOKEN_PROGRAM_ID
      })
      .remainingAccounts(remaining_accounts)
      .signers([user])
      .rpc({ skipPreflight: true })

    assert.equal((await getAccount(connection, accountBase)).amount, 600000n)
    assert.equal((await getAccount(connection, secondAccountBase)).amount, 800000n - 1990n)
  })

  it('gets vault indexes from statement', async () => {
    const data = (await connection.getAccountInfo(statement))?.data
    statement_acc = StatementAccount.load(data as Buffer)

    const vault_ids = statement_acc.vaults_to_refresh(2)

    assert.deepEqual(vault_ids, [0, 2, 1])
  })
})
