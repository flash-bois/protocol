import * as anchor from '@coral-xyz/anchor'
import { Program, BN } from '@coral-xyz/anchor'
import {
  ComputeBudgetProgram,
  Keypair,
  PublicKey,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
} from '@solana/web3.js'
import { assert } from 'chai'
import { StatementAccount, VaultsAccount } from '../../pkg/protocol'
import { Protocol } from '../../target/types/protocol'
import { Oracle } from '../../target/types/oracle'
import {
  createAssociatedTokenAccount,
  TOKEN_PROGRAM_ID,
  mintTo,
  getAccount
} from '@solana/spl-token'
import {
  waitFor,
  createTestEnvironment,
  TestEnvironment,
  IVaultAccounts
} from '../utils/utils'
import { STATEMENT_SEED } from '../../microSdk'

const provider = anchor.AnchorProvider.env()
const program = anchor.workspace.Protocol as Program<Protocol>
const oracle_program = anchor.workspace.Oracle as Program<Oracle>
const minter = Keypair.generate()
const admin = Keypair.generate()
const user = Keypair.generate()
const connection = program.provider.connection
anchor.setProvider(provider)

let test_environment: TestEnvironment
let accountBase: PublicKey
let accountQuote: PublicKey
let vaults_account: VaultsAccount
let statement_account: StatementAccount
let vault0: IVaultAccounts

const [statement_address, bump] = PublicKey.findProgramAddressSync(
  [Buffer.from(anchor.utils.bytes.utf8.encode(STATEMENT_SEED)), user.publicKey.toBuffer()],
  program.programId
)

describe('Borrow tests', function () {

  before(async function () {
    const admin_sig = await connection.requestAirdrop(admin.publicKey, 10000000000)
    await waitFor(connection, admin_sig)

    const user_sig = await connection.requestAirdrop(user.publicKey, 1000000000)
    await waitFor(connection, user_sig)


    test_environment = await createTestEnvironment({
      admin,
      minter: minter.publicKey,
      oracle_program,
      program,
      vaults_infos: [{
        base_oracle: {
          base: true, decimals: 6, skip_init: false,
          price: new BN(200000000), exp: -8, conf: new BN(200000)
        }, quote_oracle: {
          base: false, decimals: 6, skip_init: false,
          price: new BN(100000000), exp: -8, conf: new BN(100000)
        },
        lending: {
          initial_fee_time: 0,
          max_borrow: new BN(10_000_000_000),
          max_utilization: 800000,
        },
        strategies: [
          { collateral_ratio: new BN(1000000), liquidation_threshold: new BN(1000000), lend: true, swap: false, trade: false }
        ]
      }, {
        base_oracle: {
          base: true, decimals: 6, skip_init: false,
          price: new BN(200000000), exp: -8, conf: new BN(200000)
        }, quote_oracle: {
          base: false, decimals: 6, skip_init: false,
          price: new BN(100000000), exp: -8, conf: new BN(100000)
        },
        lending: {
          initial_fee_time: 0,
          max_borrow: new BN(10_000_000_000),
          max_utilization: 800000
        },
        strategies: [
          { collateral_ratio: new BN(1000000), liquidation_threshold: new BN(1000000), lend: true, swap: false, trade: false }
        ]
      }]
    })


    vault0 = test_environment.vaults_data[0]

    accountBase = await createAssociatedTokenAccount(
      connection,
      user,
      vault0.base,
      user.publicKey
    )

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


  it('oracles data from vaults equals to defined', async function () {
    const data = (await connection.getAccountInfo(test_environment.vaults))?.data
    assert.notEqual(data, undefined)
    vaults_account = VaultsAccount.load(data as Buffer)


    // BASE
    assert.equal(vaults_account.vaults_len(), 2)
    assert.equal(vaults_account.base_oracle_enabled(0), true)
    assert.equal(
      Buffer.from(vaults_account.oracle_base(0)).toString('hex'),
      test_environment.vaults_data[0].base_oracle?.toBuffer().toString('hex')
    )
    assert.equal(vaults_account.get_price(0), 2000000000n)
    assert.equal(vaults_account.get_confidence(0), 2000000n)

    // QUOTE
    assert.equal(vaults_account.quote_oracle_enabled(0), true)
    assert.equal(
      Buffer.from(vaults_account.oracle_quote(0)).toString('hex'),
      test_environment.vaults_data[0].quote_oracle?.toBuffer().toString('hex')
    )
    assert.equal(vaults_account.get_price_quote(0), 1000000000n)
    assert.equal(vaults_account.get_confidence_quote(0), 1000000n)
  })


  it('Set lend fee', async function () {
    let sig = await program.methods
      .modifyFeeCurve(0, 1, true, new BN(1000000), new BN(0), new BN(0), new BN(100))
      .accounts({
        admin: admin.publicKey,
        ...test_environment
      })
      .signers([admin])
      .rpc({ skipPreflight: true })

    await waitFor(connection, sig)
  })


  it('Creates statement', async () => {
    const sig = await program.methods
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


  it('vault 0: deposit', async () => {
    const remaining_accounts = vault0.remaining_accounts;

    const sig = await program.methods
      .deposit(0, 0, new BN(200000), true)
      .accountsStrict({
        ...test_environment,
        accountBase,
        accountQuote,
        statement: statement_address,
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
    assert.equal((await getAccount(connection, vault0.reserveBase)).amount, 200000n)
    assert.equal((await getAccount(connection, vault0.reserveQuote)).amount, 400000n)
  })


  it('borrows 100000 token units', async () => {
    const remaining_accounts = vault0.remaining_accounts;

    const sig = await program.methods
      .borrow(0, new BN(100000))
      .accountsStrict({
        ...test_environment,
        accountBase,
        statement: statement_address,
        signer: user.publicKey,
        reserveBase: vault0.reserveBase,
        tokenProgram: TOKEN_PROGRAM_ID
      })
      .remainingAccounts(remaining_accounts ?? [])
      .signers([user])
      .rpc({ skipPreflight: true })

    await waitFor(connection, sig)

    assert.equal((await getAccount(connection, accountBase)).amount, 900000n)
    assert.equal((await getAccount(connection, vault0.reserveBase)).amount, 100000n)
  })

  it('gets borrow position info', async () => {
    const statement_data = (await connection.getAccountInfo(statement_address))?.data
    statement_account = StatementAccount.load(statement_data as Buffer)

    const vaults_data = (await connection.getAccountInfo(test_environment.vaults))?.data;
    vaults_account.reload(vaults_data as Buffer)

    const borrow_position = vaults_account.get_borrow_position_info(0, statement_account.buffer(), 0)

    assert.equal(borrow_position.owed_quantity, 100000n)
    assert.equal(borrow_position.borrowed_quantity, 100000n)
  })

  it('repays 100000 token units', async () => {
    const remaining_accounts = vault0.remaining_accounts;

    const sig = await program.methods
      .repay(0, new BN(100000))
      .accountsStrict({
        ...test_environment,
        accountBase,
        statement: statement_address,
        signer: user.publicKey,
        reserveBase: vault0.reserveBase,
        tokenProgram: TOKEN_PROGRAM_ID
      })
      .preInstructions([
        ComputeBudgetProgram.setComputeUnitLimit({
          units: 1000000
        })
      ])
      .remainingAccounts(remaining_accounts ?? [])
      .signers([user])
      .rpc({ skipPreflight: true })

    await waitFor(connection, sig)

    assert.equal((await getAccount(connection, accountBase)).amount, 800000n)
    assert.equal((await getAccount(connection, vault0.reserveBase)).amount, 200000n)
  })
})