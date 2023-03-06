import * as anchor from '@coral-xyz/anchor'
import { Program } from '@coral-xyz/anchor'
import { Keypair, PublicKey, SIGNATURE_LENGTH_IN_BYTES, SystemProgram, SYSVAR_RENT_PUBKEY, Transaction } from '@solana/web3.js'
import { assert, use } from 'chai'
import { price_denominator, StateAccount, StatementAccount, VaultsAccount } from '../../pkg/protocol'
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
  DotWaveAccounts
} from '../utils/utils'
import { BN } from 'bn.js'
import { STATEMENT_SEED, STATE_SEED } from '../../microSdk'

describe('User', () => {
  const provider = anchor.AnchorProvider.env()
  const program = anchor.workspace.Protocol as Program<Protocol>

  const minter = Keypair.generate()
  const admin = Keypair.generate()
  const user = Keypair.generate()

  const connection = program.provider.connection

  anchor.setProvider(provider)

  let state: PublicKey
  let vaults: PublicKey
  let accounts: AdminAccounts
  let vaults_acc: VaultsAccount
  let statement_acc: StatementAccount
  let protocolAccounts: DotWaveAccounts
  let accountBase: PublicKey
  let accountQuote: PublicKey

  const [statement, bump] = PublicKey.findProgramAddressSync(
    [Buffer.from(anchor.utils.bytes.utf8.encode(STATEMENT_SEED)), user.publicKey.toBuffer()],
    program.programId
  )

  before(async () => {
    const sigAdmin = await connection.requestAirdrop(admin.publicKey, 1e9)
    const sigUser = await connection.requestAirdrop(user.publicKey, 1e9)
    await waitFor(connection, sigAdmin)
    await waitFor(connection, sigUser)

    const initializedProtocolAccounts = await createBasicVault(program, admin, minter)
    protocolAccounts = initializedProtocolAccounts

    state = protocolAccounts.state
    vaults = protocolAccounts.vaults
    accounts = {
      state,
      vaults,
      admin: admin.publicKey
    }

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

    await Promise.all([
      mintTo(connection, user, protocolAccounts.base, accountBase, minter, 1e6),
      mintTo(connection, user, protocolAccounts.quote, accountQuote, minter, 1e6)
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
    const account_info = (await connection.getAccountInfo(vaults))?.data
    vaults_acc = VaultsAccount.load(account_info as Buffer)

    assert.equal(vaults_acc.deposit(0, 0, 200000n, true, 0), 400000n)
    assert.equal(vaults_acc.deposit(0, 0, 400000n, false, 0), 200000n)
  })

  it('deposit', async () => {
    assert.equal((await getAccount(connection, accountBase)).amount, 1000000n)
    assert.equal((await getAccount(connection, accountQuote)).amount, 1000000n)

    const sig = await program.methods
      .deposit(0, 0, new BN(200000), true)
      .accountsStrict({
        state,
        vaults,
        accountBase,
        accountQuote,
        statement,
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
  })

  it('gets lp position info from statement', async () => {
    const account_info = (await connection.getAccountInfo(statement))?.data
    statement_acc = StatementAccount.load(account_info as Buffer)
    const vaults_acc_info = (await connection.getAccountInfo(vaults))?.data;
    vaults_acc.reload(vaults_acc_info as Buffer)

    const position_info = vaults_acc.get_lp_position_info(0, 0, statement_acc.buffer(), 0)

    assert.equal(position_info.base_quantity, 200000n)
    assert.equal(position_info.quote_quantity, 400000n)
    assert.equal(position_info.position_value, 800000000n)
  })

  it('gives max borrow user value', async () => {
    const account_info = (await connection.getAccountInfo(statement))?.data
    const vaults_acc_info = (await connection.getAccountInfo(vaults))?.data;
    vaults_acc.reload(vaults_acc_info as Buffer)
    statement_acc.reload(account_info as Buffer)

    assert.equal(Buffer.from(statement_acc.owner()).toString('hex'), user.publicKey.toBuffer().toString('hex'))

    statement_acc.refresh(vaults_acc.buffer())

    assert.equal(statement_acc.remaining_permitted_debt(), 800000000n)
  })

  it('single swap', async () => {
    assert.equal((await getAccount(connection, accountBase)).amount, 800000n)
    assert.equal((await getAccount(connection, accountQuote)).amount, 600000n)

    await program.methods
      .modifyFeeCurve(0, 2, true, new BN(1000000), new BN(0), new BN(0), new BN(10000))
      .accounts(accounts)
      .signers([admin])
      .rpc({ skipPreflight: true })

    await program.methods
      .singleSwap(0, new BN(100000), new BN(10), true, false)
      .accountsStrict({
        state,
        vaults,
        accountBase,
        accountQuote,
        signer: user.publicKey,
        reserveBase: protocolAccounts.reserveBase,
        reserveQuote: protocolAccounts.reserveQuote,
        tokenProgram: TOKEN_PROGRAM_ID
      })
      .signers([user])
      .rpc({ skipPreflight: true })

    assert.equal((await getAccount(connection, accountBase)).amount, 700000n)
    assert.equal((await getAccount(connection, accountQuote)).amount, 800000n - 2000n)
  })

  it('double swap', async () => {
    const secondProtocolAccounts = await createBasicVault(
      program,
      admin,
      minter,
      protocolAccounts,
      1
    )

    assert.equal(secondProtocolAccounts.quote.toBase58(), protocolAccounts.quote.toBase58())
    assert.notEqual(secondProtocolAccounts.base.toBase58(), protocolAccounts.base.toBase58())

    const accountInfo = await connection.getAccountInfo(vaults)
    assert.isNotNull(accountInfo)
    const vaultsAccount = VaultsAccount.load(accountInfo!.data)
    assert.equal(
      new PublicKey(vaultsAccount.base_reserve(1)).toString(),
      secondProtocolAccounts.reserveBase.toString()
    )

    const secondAccountBase = await createAssociatedTokenAccount(
      connection,
      user,
      secondProtocolAccounts.base,
      user.publicKey
    )

    await mintTo(connection, user, secondProtocolAccounts.base, secondAccountBase, minter, 1e6)

    await program.methods
      .deposit(1, 0, new BN(300000), true)
      .accountsStrict({
        state,
        vaults,
        accountBase: secondAccountBase,
        accountQuote,
        statement,
        signer: user.publicKey,
        reserveBase: secondProtocolAccounts.reserveBase,
        reserveQuote: secondProtocolAccounts.reserveQuote,
        tokenProgram: TOKEN_PROGRAM_ID
      })
      .signers([user])
      .rpc({ skipPreflight: true })

    await program.methods
      .modifyFeeCurve(1, 2, false, new BN(1000000), new BN(0), new BN(0), new BN(10000))
      .accounts(accounts)
      .signers([admin])
      .rpc({ skipPreflight: true })

    assert.equal((await getAccount(connection, accountBase)).amount, 700000n)
    assert.equal((await getAccount(connection, secondAccountBase)).amount, 700000n)

    await program.methods
      .doubleSwap(0, 1, new BN(100000), new BN(10), false)
      .accountsStrict({
        state,
        vaults,
        accountIn: accountBase,
        accountOut: secondAccountBase,
        signer: user.publicKey,
        reserveIn: protocolAccounts.reserveBase,
        reserveInQuote: protocolAccounts.reserveQuote,
        reserveOut: secondProtocolAccounts.reserveBase,
        reserveOutQuote: secondProtocolAccounts.reserveQuote,
        tokenProgram: TOKEN_PROGRAM_ID
      })
      .signers([user])
      .rpc({ skipPreflight: true })

    assert.equal((await getAccount(connection, accountBase)).amount, 600000n)
    assert.equal((await getAccount(connection, secondAccountBase)).amount, 800000n - 1990n)
  })
})
