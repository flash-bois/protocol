import * as anchor from '@coral-xyz/anchor'
import { Program } from '@coral-xyz/anchor'
import { Keypair, PublicKey, SystemProgram, SYSVAR_RENT_PUBKEY, Transaction } from '@solana/web3.js'
import { assert } from 'chai'
import { StateAccount, VaultsAccount } from '../../pkg/protocol'
import { Protocol } from '../../target/types/protocol'
import { createMint, createAssociatedTokenAccount, TOKEN_PROGRAM_ID } from '@solana/spl-token'
import { init_accounts } from '../utils/utils'

const STATE_SEED = 'state'

describe('Init vault', () => {
  const provider = anchor.AnchorProvider.env()
  const program = anchor.workspace.Protocol as Program<Protocol>

  const minter = Keypair.generate()
  const admin = Keypair.generate()
  const user = Keypair.generate()

  const connection = program.provider.connection

  anchor.setProvider(provider)

  it('Creates vault', async () => {
    await connection.requestAirdrop(admin.publicKey, 1000000000)
    const { blockhash, lastValidBlockHeight } = await connection.getLatestBlockhash()

    const { state, vaults } = await init_accounts(connection, program, admin)

    const base = await createMint(connection, admin, minter.publicKey, null, 6)
    const quote = await createMint(connection, admin, minter.publicKey, null, 6)

    const reserveBase = Keypair.generate()
    const reserveQuote = Keypair.generate()

    await program.methods
      .initVault()
      .accounts({
        state,
        vaults,
        base,
        quote,
        reserveBase: reserveBase.publicKey,
        reserveQuote: reserveQuote.publicKey,
        admin: admin.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId
      })
      .signers([admin, reserveBase, reserveQuote])
      .rpc({ skipPreflight: true })

    let vault_account_info = (await connection.getAccountInfo(vaults))?.data
    assert.notEqual(vault_account_info, undefined)

    if (vault_account_info) {
      const vaultsAccount = VaultsAccount.load(vault_account_info)
      assert.equal(vaultsAccount.vaults_len(), 1)
    }
  })
})
