import * as anchor from '@coral-xyz/anchor'
import { Program, BN } from '@coral-xyz/anchor'
import {
  ComputeBudgetProgram,
  Keypair,
  PublicKey,
  SystemProgram,
  SYSVAR_RENT_PUBKEY
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
import { waitFor, createTestEnvironment, TestEnvironment, IVaultAccounts } from '../utils/utils'
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
let vault: IVaultAccounts

const [statement_address, bump] = PublicKey.findProgramAddressSync(
  [Buffer.from(anchor.utils.bytes.utf8.encode(STATEMENT_SEED)), user.publicKey.toBuffer()],
  program.programId
)

describe('Trading tests', function () {
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
      vaults_infos: [
        {
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
          trading: {
            collateral_ratio: 1000000,
            liquidation_threshold: 1000000,
            max_leverage: 5000000,
            open_fee: 1000
          },
          strategies: [
            {
              collateral_ratio: new BN(1000000),
              liquidation_threshold: new BN(1000000),
              lend: false,
              swap: false,
              trade: true
            }
          ]
        }
      ]
    })

    vault = test_environment.vaults_data[0]

    accountBase = await createAssociatedTokenAccount(connection, user, vault.base, user.publicKey)
    accountQuote = await createAssociatedTokenAccount(connection, user, vault.quote, user.publicKey)

    await Promise.all([
      mintTo(connection, user, vault.base, accountBase, minter, 1e9),
      mintTo(connection, user, vault.quote, accountQuote, minter, 1e9)
    ])
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

  it('Vault: deposit', async () => {
    const remaining_accounts = vault.remaining_accounts

    const sig = await program.methods
      .deposit(0, 0, new BN(200000000), true)
      .accountsStrict({
        ...test_environment,
        accountBase,
        accountQuote,
        statement: statement_address,
        signer: user.publicKey,
        reserveBase: vault.reserveBase,
        reserveQuote: vault.reserveQuote,
        tokenProgram: TOKEN_PROGRAM_ID
      })
      .signers([user])
      .remainingAccounts(remaining_accounts ?? [])
      .rpc({ skipPreflight: true })

    await waitFor(connection, sig)

    assert.equal((await getAccount(connection, accountBase)).amount, 800000000n)
    assert.equal((await getAccount(connection, accountQuote)).amount, 600000000n)
    assert.equal((await getAccount(connection, vault.reserveBase)).amount, 200000000n)
    assert.equal((await getAccount(connection, vault.reserveQuote)).amount, 400000000n)
  })

  it('Open Long', async () => {
    const remaining_accounts = vault.remaining_accounts

    const sig = await program.methods
      .openPosition(0, new BN(2000000), true)
      .accounts({
        ...test_environment,
        accountBase,
        accountQuote,
        statement: statement_address,
        signer: user.publicKey,
        reserveBase: vault.reserveBase,
        reserveQuote: vault.reserveQuote,
        tokenProgram: TOKEN_PROGRAM_ID
      })
      .signers([user])
      .remainingAccounts(remaining_accounts ?? [])
      .rpc({ skipPreflight: true })

    await waitFor(connection, sig)

    assert.equal((await getAccount(connection, accountBase)).amount, 800000000n)
    assert.equal((await getAccount(connection, accountQuote)).amount, 600000000n)
    assert.equal((await getAccount(connection, vault.reserveBase)).amount, 200000000n)
    assert.equal((await getAccount(connection, vault.reserveQuote)).amount, 400000000n)
  })

  it('gets trading position info', async () => {
    const statement_data = (await connection.getAccountInfo(statement_address))?.data
    statement_account = StatementAccount.load(statement_data as Buffer)

    const vaults_data = (await connection.getAccountInfo(test_environment.vaults))?.data
    vaults_account = VaultsAccount.load(vaults_data as Buffer)

    const trading_position_info = vaults_account.get_trading_position_info(0, statement_account.buffer(), 0)

    console.log(trading_position_info.fees)
    console.log(trading_position_info.fees_value)
  })

  // it('borrows 100000 token units', async () => {
  //   const remaining_accounts = vault0.remaining_accounts;

  //   const sig = await program.methods
  //     .borrow(0, new BN(100000))
  //     .accountsStrict({
  //       ...test_environment,
  //       accountBase,
  //       statement: statement_address,
  //       signer: user.publicKey,
  //       reserveBase: vault0.reserveBase,
  //       tokenProgram: TOKEN_PROGRAM_ID
  //     })
  //     .remainingAccounts(remaining_accounts ?? [])
  //     .signers([user])
  //     .rpc({ skipPreflight: true })

  //   await waitFor(connection, sig)

  //   assert.equal((await getAccount(connection, accountBase)).amount, 900000n)
  //   assert.equal((await getAccount(connection, vault0.reserveBase)).amount, 100000n)
  // })

  // it('gets borrow position info', async () => {
  //   const statement_data = (await connection.getAccountInfo(statement_address))?.data
  //   statement_account.reload(statement_data as Buffer)

  //   const vaults_data = (await connection.getAccountInfo(test_environment.vaults))?.data;
  //   vaults_account.reload(vaults_data as Buffer)

  //   const borrow_position = vaults_account.get_borrow_position_info(0, statement_account.buffer(), 0)

  //   assert.equal(borrow_position.owed_quantity, 100000n)
  //   assert.equal(borrow_position.borrowed_quantity, 100000n)
  // })

  // it('repays 100000 token units', async () => {
  //   const remaining_accounts = vault0.remaining_accounts;

  //   const sig = await program.methods
  //     .repay(0, new BN(100000))
  //     .accountsStrict({
  //       ...test_environment,
  //       accountBase,
  //       statement: statement_address,
  //       signer: user.publicKey,
  //       reserveBase: vault0.reserveBase,
  //       tokenProgram: TOKEN_PROGRAM_ID
  //     })
  //     .preInstructions([
  //       ComputeBudgetProgram.setComputeUnitLimit({
  //         units: 1000000
  //       })
  //     ])
  //     .remainingAccounts(remaining_accounts ?? [])
  //     .signers([user])
  //     .rpc({ skipPreflight: true })

  //   await waitFor(connection, sig)

  //   assert.equal((await getAccount(connection, accountBase)).amount, 800000n)
  //   assert.equal((await getAccount(connection, vault0.reserveBase)).amount, 200000n)
  // })
})
