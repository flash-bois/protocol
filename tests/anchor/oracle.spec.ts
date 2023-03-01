import * as anchor from '@coral-xyz/anchor'
import { Program } from '@coral-xyz/anchor'
import { Keypair, PublicKey, SystemProgram, SYSVAR_RENT_PUBKEY, Transaction } from '@solana/web3.js'
import { assert } from 'chai'
import { price_denominator, StateAccount, VaultsAccount } from '../../pkg/protocol'
import { Protocol } from '../../target/types/protocol'
import { Oracle } from '../../target/types/oracle'
import {
  createMint,
  createAssociatedTokenAccount,
  TOKEN_PROGRAM_ID,
  createAccount
} from '@solana/spl-token'
import { createAccounts, initAccounts, sleep, waitFor } from '../utils/utils'
import { SOL_ORACLE } from '../../microSdk'
import { BN } from 'bn.js'

const STATE_SEED = 'state'

describe('Enable Oracle', () => {
  const provider = anchor.AnchorProvider.env()
  const program = anchor.workspace.Protocol as Program<Protocol>
  const oracle_program = anchor.workspace.Oracle as Program<Oracle>
  const minter = Keypair.generate()
  const admin = Keypair.generate()
  const user = Keypair.generate()

  const connection = program.provider.connection

  anchor.setProvider(provider)

  let state: PublicKey
  let vaults: PublicKey
  let local_oracle: PublicKey

  before(async () => {
    const sig = await connection.requestAirdrop(admin.publicKey, 1000000000)
    await waitFor(connection, sig)

    const { state: s, vaults: v } = await initAccounts(program, admin, minter)
    state = s
    vaults = v
  })

  it('before oracle', async () => {
    let data = (await connection.getAccountInfo(vaults))?.data
    assert.notEqual(data, undefined)

    if (data) {
      const vaultsAccount = VaultsAccount.load(data)
      assert.equal(vaultsAccount.vaults_len(), 1)
      assert.equal(vaultsAccount.base_oracle_enabled(0), false)
      assert.equal(vaultsAccount.quote_oracle_enabled(0), false)
    }
  })

  it('creates local oracle', async () => {
    let oracle = Keypair.generate()

    console.log(oracle_program.programId.toString())
    const sig = await oracle_program.methods
      .set(new BN(2200000000), -8, new BN(2200000000))
      .preInstructions([
        SystemProgram.createAccount({
          fromPubkey: admin.publicKey,
          newAccountPubkey: oracle.publicKey,
          space: 3312,
          lamports: await connection.getMinimumBalanceForRentExemption(3312),
          programId: oracle_program.programId
        })
      ])
      .accounts({ price: oracle.publicKey })
      .signers([oracle, admin])
      .rpc()

    await waitFor(oracle_program.provider.connection, sig)

    local_oracle = oracle.publicKey
  })

  it('enable base oracle', async () => {
    // const priceFeed = Keypair.generate().publicKey

    const sig = await program.methods
      .enableOracle(0, 6, true, false)
      .preInstructions([])
      .accounts({
        state,
        vaults,
        admin: admin.publicKey,
        priceFeed: local_oracle
      })
      .signers([admin])
      .rpc({ skipPreflight: true })

    await waitFor(connection, sig)

    let data = (await connection.getAccountInfo(vaults))?.data
    assert.notEqual(data, undefined)

    if (data) {
      const vaultsAccount = VaultsAccount.load(data)
      assert.equal(vaultsAccount.vaults_len(), 1)
      assert.equal(vaultsAccount.base_oracle_enabled(0), true)
      assert.equal(vaultsAccount.quote_oracle_enabled(0), false)
      assert.equal(
        Buffer.from(vaultsAccount.oracle_base(0)).toString('hex'),
        local_oracle.toBuffer().toString('hex')
      )
    }
  })

  it('enable base oracle', async () => {
    const quotePriceFeed = Keypair.generate().publicKey

    const otherSig = await program.methods
      .enableOracle(0, 6, false, true)
      .accounts({
        state,
        vaults,
        admin: admin.publicKey,
        priceFeed: quotePriceFeed
      })
      .signers([admin])
      .rpc({ skipPreflight: true })

    await waitFor(connection, otherSig)

    let data = (await connection.getAccountInfo(vaults))?.data
    assert.notEqual(data, undefined)

    if (data) {
      const vaultsAccount = VaultsAccount.load(data)
      assert.equal(vaultsAccount.vaults_len(), 1)
      assert.equal(vaultsAccount.base_oracle_enabled(0), true)
      assert.equal(vaultsAccount.quote_oracle_enabled(0), true)
      assert.equal(
        Buffer.from(vaultsAccount.oracle_quote(0)).toString('hex'),
        quotePriceFeed.toBuffer().toString('hex')
      )
    }
  })

  it('force override oracle', async () => {
    await program.methods
      .forceOverrideOracle(0, true, 200, 1, -2, null)
      .accounts({ state, vaults, admin: admin.publicKey })
      .signers([admin])
      .rpc({ skipPreflight: true })

    let data = (await connection.getAccountInfo(vaults))?.data
    assert.notEqual(data, undefined)

    if (data) {
      const vaultsAccount = VaultsAccount.load(data)
      assert.equal(price_denominator(), 1000000000n)
      assert.equal(vaultsAccount.get_price(0), 2000000000n)
      assert.equal(vaultsAccount.get_confidence(0), 10000000n)
    }
  })

  it('force override quote oracle', async () => {
    await program.methods
      .forceOverrideOracle(0, false, 200, 1, -2, 42)
      .accounts({ state, vaults, admin: admin.publicKey })
      .signers([admin])
      .rpc({ skipPreflight: true })

    let data = (await connection.getAccountInfo(vaults))?.data
    assert.notEqual(data, undefined)

    if (data) {
      const vaultsAccount = VaultsAccount.load(data)
      assert.equal(vaultsAccount.get_price_quote(0), 2000000000n)
      assert.equal(vaultsAccount.get_confidence_quote(0), 10000000n)
    }
  })

  it('locally update oracle', async () => {
    let data = (await connection.getAccountInfo(vaults))?.data
    assert.notEqual(data, undefined)

    if (data) {
      const vaultsAccount = VaultsAccount.load(data)

      vaultsAccount.update_oracle(0, 3000000000n, 15000000n, 0)
      assert.equal(vaultsAccount.get_price(0), 3000000000n)
      assert.equal(vaultsAccount.get_confidence(0), 15000000n)

      vaultsAccount.update_oracle(0, 5000000000n, 25000000n, 0)
      assert.equal(vaultsAccount.get_price(0), 5000000000n)
      assert.equal(vaultsAccount.get_confidence(0), 25000000n)
    }
  })
})
