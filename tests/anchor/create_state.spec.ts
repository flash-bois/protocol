import * as anchor from '@coral-xyz/anchor'
import { Program } from '@coral-xyz/anchor'
import { Keypair, PublicKey, SystemProgram, SYSVAR_RENT_PUBKEY, Transaction } from '@solana/web3.js'
import { assert } from 'chai'
import { StateAccount, VaultsAccount } from '../../pkg/protocol'
import { Protocol } from '../../target/types/protocol'
import { STATE_SEED } from '../../microSdk'

describe('state with default vaults', () => {
  const provider = anchor.AnchorProvider.env()
  const program = anchor.workspace.Protocol as Program<Protocol>

  const admin = Keypair.generate()
  const vaults = Keypair.generate()

  const vaults_size = program.account.vaults.size
  const connection = program.provider.connection

  anchor.setProvider(provider)

  const [state_address, bump] = PublicKey.findProgramAddressSync(
    [Buffer.from(anchor.utils.bytes.utf8.encode(STATE_SEED))],
    program.programId
  )

  it('Creates state', async () => {
    const airdrop_signature = await connection.requestAirdrop(admin.publicKey, 1000000000)
    const { blockhash, lastValidBlockHeight } = await connection.getLatestBlockhash()

    await program.provider.connection.confirmTransaction({
      signature: airdrop_signature,
      blockhash,
      lastValidBlockHeight
    })

    const create_vaults_account_ix = SystemProgram.createAccount({
      fromPubkey: admin.publicKey,
      newAccountPubkey: vaults.publicKey,
      space: VaultsAccount.size(),
      lamports: await provider.connection.getMinimumBalanceForRentExemption(VaultsAccount.size()),
      programId: program.programId
    })

    await program.methods
      .createState()
      .accounts({
        admin: admin.publicKey,
        state: state_address,
        rent: SYSVAR_RENT_PUBKEY,
        systemProgram: SystemProgram.programId,
        vaults: vaults.publicKey
      })
      .preInstructions([create_vaults_account_ix])
      .signers([admin, vaults])
      .rpc({ skipPreflight: true })

    let account_info = (await connection.getAccountInfo(state_address))?.data
    assert.notEqual(account_info, undefined)
    if (account_info) {
      const state = StateAccount.load(account_info)
      assert.equal(state.get_bump(), bump)
    }

    let vault_account_info = (await connection.getAccountInfo(vaults.publicKey))?.data
    assert.notEqual(vault_account_info, undefined)
    if (vault_account_info) {
      const state = VaultsAccount.load(vault_account_info)
      assert.equal(state.vaults_len(), 0)
    }
  })
})
