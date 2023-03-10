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
  IVaultAccounts,
  deposit,
  IUserAccounts,
  withdraw
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
let user_accounts: IUserAccounts

const [statement, bump] = PublicKey.findProgramAddressSync(
  [Buffer.from(anchor.utils.bytes.utf8.encode(STATEMENT_SEED)), user.publicKey.toBuffer()],
  program.programId
)

describe('Deposit and withdraw', () => {
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
          lending: {
            initial_fee_time: 0,
            max_borrow: new BN(10_000_000_000),
            max_utilization: 800000
          },
          strategies: [
            {
              collateral_ratio: new BN(1000000),
              liquidation_threshold: new BN(1000000),
              lend: true,
              swap: false,
              trade: false
            }
          ]
        }
      ]
    })

    vault0 = test_environment.vaults_data[0]

    accountBase = await createAssociatedTokenAccount(connection, user, vault0.base, user.publicKey)

    accountQuote = await createAssociatedTokenAccount(
      connection,
      user,
      vault0.quote,
      user.publicKey
    )

    await Promise.all([
      mintTo(connection, user, vault0.base, accountBase, minter, 1e10),
      mintTo(connection, user, vault0.quote, accountQuote, minter, 1e10)
    ])

    user_accounts = {
      accountBase,
      accountQuote,
      statement,
      user
    }
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

  it('deposit', async () => {
    const remaining_accounts = vault0.remaining_accounts

    assert.equal((await getAccount(connection, accountBase)).amount, 10000000000n)
    assert.equal((await getAccount(connection, accountQuote)).amount, 10000000000n)

    await deposit({
      ...test_environment,
      ...vault0,
      ...user_accounts,
      program,
      strategy: 0,
      vault: 0,
      base: true,
      amount: new BN(2000000000),
      remaining_accounts: remaining_accounts!
    })

    assert.equal((await getAccount(connection, accountBase)).amount, 8000000000n)
    assert.equal((await getAccount(connection, accountQuote)).amount, 6000000000n)
  })

  it('withdraw', async () => {
    const remaining_accounts = vault0.remaining_accounts

    console.log(remaining_accounts![0].pubkey.toString() + ' ' + remaining_accounts![1].pubkey.toString())

    await withdraw({
      ...test_environment,
      ...vault0,
      ...user_accounts,
      program,
      strategy: 0,
      vault: 0,
      base: true,
      amount: new BN(2000000000),
      remaining_accounts: remaining_accounts!
    })

    assert.equal((await getAccount(connection, accountBase)).amount, 10000000000n)
    assert.equal((await getAccount(connection, accountQuote)).amount, 10000000000n)
  })
})
