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
  createAccount
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
  STATEMENT_SEED,
  DotWaveAccounts
} from '../utils/utils'
import { BN } from 'bn.js'

const STATE_SEED = 'state'

describe('Services', () => {
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
    const sigAdmin = await connection.requestAirdrop(admin.publicKey, 1000000000)
    const sigUser = await connection.requestAirdrop(user.publicKey, 1000000000)
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

    const { accountBase: b, accountQuote: q } = await mintTokensForUser(
      connection,
      minter,
      user,
      protocolAccounts.base,
      protocolAccounts.quote
    )
    accountBase = b
    accountQuote = q
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
    await program.methods
      .deposit(0, 0, new BN(1000000), true)
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
  })

  it('single swap', async () => {
    await program.methods
      .singleSwap(0, new BN(100), new BN(10), true, false)
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
  })
})
