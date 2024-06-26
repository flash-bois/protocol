import * as anchor from '@coral-xyz/anchor'
import { Program } from '@coral-xyz/anchor'
import { Keypair, PublicKey, SystemProgram, SYSVAR_RENT_PUBKEY, Transaction } from '@solana/web3.js'
import { assert } from 'chai'
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
  enableTrading
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

  before(async () => {
    const sig = await connection.requestAirdrop(admin.publicKey, 1000000000)
    await waitFor(connection, sig)

    const { state: s, vaults: v } = await initAccounts(program, admin, minter)
    state = s
    vaults = v
    accounts = {
      state,
      vaults,
      admin: admin.publicKey
    }

    await enableOracles(program, 0, accounts, admin)
  })

  it('enable lend', async () => {
    await program.methods
      .enableLending(0, 800000, new BN(10000_000000), 0)
      .accounts(accounts)
      .signers([admin])
      .rpc({ skipPreflight: true })

    let data = (await connection.getAccountInfo(vaults))?.data
    assert.notEqual(data, undefined)

    if (data) {
      const vaultsAccount = VaultsAccount.load(data)
      assert.equal(vaultsAccount.vaults_len(), 1)
      assert.equal(vaultsAccount.has_lending(0), true)
      assert.equal(vaultsAccount.has_swap(0), false)
      assert.equal(vaultsAccount.has_trading(0), false)
    }
  })

  it('enable swap', async () => {
    await program.methods
      .enableSwapping(0, 100000, new BN(10000_000000))
      .accounts(accounts)
      .signers([admin])
      .rpc({ skipPreflight: true })

    const vaultsAccount = VaultsAccount.load(await tryFetch(connection, vaults))

    assert.equal(vaultsAccount.has_lending(0), true)
    assert.equal(vaultsAccount.has_swap(0), true)
    assert.equal(vaultsAccount.has_trading(0), false)
  })

  it('enable trade', async () => {
    await enableTrading({
      ix_only: false,
      program,
      admin,
      state: accounts.state,
      vaults: accounts.vaults,
      vault: 0,
      collateral_ratio: 100000,
      liquidation_threshold: 1000000,
      max_leverage: 3000000,
      open_fee: 20000
    })

    const vaultsAccount = VaultsAccount.load(await tryFetch(connection, vaults))

    assert.equal(vaultsAccount.has_lending(0), true)
    assert.equal(vaultsAccount.has_swap(0), true)
    assert.equal(vaultsAccount.has_trading(0), true)
  })

  it('set swap fee', async () => {
    await program.methods
      .modifyFeeCurve(0, 2, true, new BN(1000000), new BN(0), new BN(0), new BN(100))
      .accounts(accounts)
      .signers([admin])
      .rpc({ skipPreflight: true })
  })

  it('add strategies', async () => {
    let vaultsAccount = VaultsAccount.load(await tryFetch(connection, vaults))

    assert.equal(vaultsAccount.has_lending(0), true)
    assert.equal(vaultsAccount.has_swap(0), true)

    await program.methods
      .addStrategy(0, true, false, false, new BN(1000000), new BN(1000000))
      .accounts(accounts)
      .signers([admin])
      .rpc({ skipPreflight: true })

    vaultsAccount = VaultsAccount.load(await tryFetch(connection, vaults))

    assert.equal(vaultsAccount.strategy_info(0, 0).has_lend, true)
    assert.equal(vaultsAccount.strategy_info(0, 0).has_swap, false)

    await program.methods
      .addStrategy(0, false, true, false, new BN(1000000), new BN(1000000))
      .accounts(accounts)
      .signers([admin])
      .rpc({ skipPreflight: true })
    vaultsAccount = VaultsAccount.load(await tryFetch(connection, vaults))

    assert.equal(vaultsAccount.strategy_info(0, 1).has_lend, false)
    assert.equal(vaultsAccount.strategy_info(0, 1).has_swap, true)

    await program.methods
      .addStrategy(0, true, true, false, new BN(1000000), new BN(1000000))
      .accounts(accounts)
      .signers([admin])
      .rpc({ skipPreflight: true })
    vaultsAccount = VaultsAccount.load(await tryFetch(connection, vaults))

    assert.equal(vaultsAccount.strategy_info(0, 2).has_swap, true)
    assert.equal(vaultsAccount.strategy_info(0, 2).has_swap, true)
  })
})
