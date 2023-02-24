import * as anchor from '@coral-xyz/anchor'
import { Program, } from '@coral-xyz/anchor'
import { Connection, Keypair, PublicKey, SystemProgram, SYSVAR_RENT_PUBKEY, Transaction } from '@solana/web3.js'
import { assert } from 'chai'
import { StateAccount, VaultsAccount } from '../../pkg/protocol'
import { Protocol } from '../../target/types/protocol'
import {createMint, createAssociatedTokenAccount} from '@solana/spl-token'


export interface DotWaveAccounts {
    state: PublicKey,
    vaults: PublicKey,
    // base: PublicKey,
    // quote: PublicKey,
    // reserveBase: PublicKey,
    // reserveQuote: PublicKey,

}

export const STATE_SEED = "state"


export async function init_accounts(connection: Connection, program: Program<Protocol>, admin: Keypair): Promise<DotWaveAccounts> {
    const vaults = Keypair.generate()
    const [state, bump] = PublicKey.findProgramAddressSync(
        [Buffer.from(anchor.utils.bytes.utf8.encode(STATE_SEED))],
        program.programId
      )

    const airdrop_signature = await connection.requestAirdrop(admin.publicKey, 1000000000)
    const { blockhash, lastValidBlockHeight } = await connection.getLatestBlockhash()

    const create_vaults_account_ix = SystemProgram.createAccount({
        fromPubkey: admin.publicKey,
        newAccountPubkey: vaults.publicKey,
        space: VaultsAccount.size(),
        lamports: await connection.getMinimumBalanceForRentExemption(
          VaultsAccount.size()
        ),
        programId: program.programId
      })

      const create_state_ix = await program.methods.createState().accounts({
        admin: admin.publicKey,
        state,
        rent: SYSVAR_RENT_PUBKEY,
        systemProgram: SystemProgram.programId,
        vaults: vaults.publicKey
      }).instruction()

      let tx = new Transaction()
        .add(create_vaults_account_ix)
        .add(create_state_ix)

      tx.recentBlockhash = blockhash
      tx.feePayer = admin.publicKey
      tx.partialSign(admin, vaults)
  
      const raw_tx = tx.serialize()
      const final_signature = await connection.sendRawTransaction(raw_tx, { skipPreflight: true })
  
     await program.provider.connection.confirmTransaction({
        blockhash,
        lastValidBlockHeight,
        signature: final_signature
      })

    return {
        state,
        vaults: vaults.publicKey,
    }
}