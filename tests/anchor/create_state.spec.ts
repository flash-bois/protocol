import * as anchor from '@coral-xyz/anchor'
import { Program, } from '@coral-xyz/anchor'
import { Keypair, PublicKey, SystemProgram, SYSVAR_RENT_PUBKEY, Transaction } from '@solana/web3.js'
import { assert } from 'chai'
import { StateAccount } from '../../pkg/protocol'
import { Protocol } from '../../target/types/protocol'

const STATE_SEED = "state"

describe('state with default vaults', () => {
  const provider = anchor.AnchorProvider.env()
  const program = anchor.workspace.Protocol as Program<Protocol>

  const admin = Keypair.generate()
  const vaults = Keypair.generate()

  const vaults_size = program.account.vaults.size;
  const connection = program.provider.connection;

  anchor.setProvider(provider)

  const [state_address, bump] = PublicKey.findProgramAddressSync(
    [Buffer.from(anchor.utils.bytes.utf8.encode(STATE_SEED))],
    program.programId
  )


  it('Creates state', async () => {
    let tx = new Transaction();

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
      space: vaults_size,
      lamports: await provider.connection.getMinimumBalanceForRentExemption(
        vaults_size
      ),
      programId: program.programId
    })

    tx.add(create_vaults_account_ix)

    const create_state_ix = await program.methods.createState().accounts({
      admin: admin.publicKey,
      state: state_address,
      rent: SYSVAR_RENT_PUBKEY,
      systemProgram: SystemProgram.programId,
      vaults: vaults.publicKey
    }).instruction()

    tx.add(create_state_ix)
    tx.recentBlockhash = blockhash
    tx.feePayer = admin.publicKey
    tx.partialSign(admin, vaults)

    const raw_tx = tx.serialize()
    const final_signature = await provider.connection.sendRawTransaction(raw_tx, { skipPreflight: true })

    await program.provider.connection.confirmTransaction({
      blockhash,
      lastValidBlockHeight,
      signature: final_signature
    })

    const vaults_account = await program.account.vaults.fetch(vaults.publicKey)
    const state_account = await program.account.state.fetch(state_address)


    // assert.equal(vaults_account.arr.head, 0)
    // assert.equal(state_account.admin.toString(), admin.publicKey.toString())
    // assert.equal(state_account.bump, bump)

    let account_info = (await connection.getAccountInfo(state_address))?.data
    console.log(account_info?.toString('hex'))

console.log(bump)
    
    if(account_info) {
     const state =  StateAccount.load(account_info)
      console.log(state.get_bump())
    }
  })
})

