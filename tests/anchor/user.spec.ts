import * as anchor from '@coral-xyz/anchor'
import { Program } from '@coral-xyz/anchor'
import { Keypair, PublicKey, SystemProgram, SYSVAR_RENT_PUBKEY, Transaction } from '@solana/web3.js'
import { assert, use } from 'chai'
import { price_denominator, StateAccount, VaultsAccount } from '../../pkg/protocol'
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
  let protocolAccounts: DotWaveAccounts
  let accountBase: PublicKey
  let accountQuote: PublicKey

  const [statement] = PublicKey.findProgramAddressSync(
    [Buffer.from(anchor.utils.bytes.utf8.encode(STATEMENT_SEED)), user.publicKey.toBuffer()],
    program.programId
  )

  before(async () => {
    const sigAdmin = await connection.requestAirdrop(admin.publicKey, 1e9)
    const sigUser = await connection.requestAirdrop(user.publicKey, 1e9)
    await waitFor(connection, sigAdmin)
    await waitFor(connection, sigUser)

    const initializedProtocolAccounts = await createBasicVault(program, admin, minter, 2)
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

  it('deposit', async () => {
    assert.equal((await getAccount(connection, accountBase)).amount, 1000000n)
    assert.equal((await getAccount(connection, accountQuote)).amount, 1000000n)

    await program.methods
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

    assert.equal((await getAccount(connection, accountBase)).amount, 800000n)
    assert.equal((await getAccount(connection, accountQuote)).amount, 600000n)
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
    const accountInfo = await connection.getAccountInfo(vaults)
    assert.notEqual(accountInfo, null)
    const vaultsAccount = VaultsAccount.load(accountInfo?.data as Buffer)
    const otherBase = new PublicKey(vaultsAccount.base_token(1))
    const otherQuote = new PublicKey(vaultsAccount.quote_token(1))
    assert.equal(otherQuote.toBase58(), protocolAccounts.quote.toBase58())
    assert.notEqual(otherBase.toBase58(), protocolAccounts.base.toBase58())

    const accountOther = await createAssociatedTokenAccount(
      connection,
      user,
      otherBase,
      user.publicKey
    )
    await mintTo(connection, user, otherBase, accountOther, minter, 1e6),
      await program.methods
        .deposit(1, 1, new BN(200000), true)
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

    await program.methods
      .modifyFeeCurve(1, 2, false, new BN(1000000), new BN(0), new BN(0), new BN(10000))
      .accounts(accounts)
      .signers([admin])
      .rpc({ skipPreflight: true })

    await program.methods
      .doubleSwap(0, 1, new BN(100000), new BN(10), false)
      .accountsStrict({
        state,
        vaults,
        accountOut: accountOther,
        accountIn: accountBase,
        signer: user.publicKey,
        reserveOut: otherBase,
        reserveIn: protocolAccounts.reserveBase,
        tokenProgram: TOKEN_PROGRAM_ID
      })
      .signers([user])
      .rpc({ skipPreflight: true })

    // TODO: uncomment and confirm this

    // assert.equal((await getAccount(connection, accountBase)).amount, 700000n)
    // assert.equal((await getAccount(connection, accountQuote)).amount, 800000n - 2000n)
  })
})
