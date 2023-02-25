import * as anchor from '@coral-xyz/anchor'
import { Program } from '@coral-xyz/anchor'
import { Keypair, PublicKey, SystemProgram, SYSVAR_RENT_PUBKEY, Transaction } from '@solana/web3.js'
import { assert } from 'chai'
import { StateAccount, VaultsAccount } from '../../pkg/protocol'
import { Protocol } from '../../target/types/protocol'
import {
  createMint,
  createAssociatedTokenAccount,
  TOKEN_PROGRAM_ID,
  createAccount
} from '@solana/spl-token'
import { createAccounts, initAccounts } from '../utils/utils'

const STATE_SEED = 'state'

describe('Enable Oracle', () => {
  const provider = anchor.AnchorProvider.env()
  const program = anchor.workspace.Protocol as Program<Protocol>

  const minter = Keypair.generate()
  const admin = Keypair.generate()
  const user = Keypair.generate()

  const connection = program.provider.connection

  anchor.setProvider(provider)

  let state: PublicKey
  let vaults: PublicKey

  it('Enables Oracle', async () => {
    await connection.requestAirdrop(admin.publicKey, 1000000000)
    const { blockhash, lastValidBlockHeight } = await connection.getLatestBlockhash()

    const { state: s, vaults: v } = await initAccounts(connection, program, admin, minter)
    state = s
    vaults = v

    const priceFeed = Keypair.generate().publicKey

    await program.methods
      .enableOracle(0, 6, true, true)
      .accounts({
        state,
        vaults,
        admin: admin.publicKey,
        priceFeed
      })
      .signers([admin])
      .rpc({ skipPreflight: true })

    let vault_account_info = (await connection.getAccountInfo(vaults))?.data
    assert.notEqual(vault_account_info, undefined)

    if (vault_account_info) {
      const vaultsAccount = VaultsAccount.load(vault_account_info)
      assert.equal(vaultsAccount.vaults_len(), 1)
    }
  })
})
