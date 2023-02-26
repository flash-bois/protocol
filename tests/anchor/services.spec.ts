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
import { createAccounts, initAccounts, sleep, waitFor } from '../utils/utils'
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

  before(async () => {
    const sig = await connection.requestAirdrop(admin.publicKey, 1000000000)
    await waitFor(connection, sig)

    const { state: s, vaults: v } = await initAccounts(connection, program, admin, minter)
    state = s
    vaults = v
  })

  it('enable lend', async () => {
    await program.methods
      .enableLending(
        0 //, 800000, new BN(10000_000000)
      )
      .accounts({
        state: state,
        vaults: vaults,
        admin: admin.publicKey
      })
      .signers([admin])
      .rpc({ skipPreflight: true })

    let data = (await connection.getAccountInfo(vaults))?.data
    assert.notEqual(data, undefined)
    if (data) {
      const vaultsAccount = VaultsAccount.load(data)
      assert.equal(vaultsAccount.vaults_len(), 1)
      assert.equal(vaultsAccount.has_lending(0), true)
      assert.equal(vaultsAccount.has_swap(0), false)
    }
  })
})
