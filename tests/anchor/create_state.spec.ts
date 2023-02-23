import * as anchor from '@coral-xyz/anchor'
import { Program, } from '@coral-xyz/anchor'
import { Keypair, PublicKey, SystemProgram, SYSVAR_RENT_PUBKEY, Transaction } from '@solana/web3.js'
import { assert } from 'chai'
import { Protocol } from '../../target/types/protocol'

const SEED = "DotWave"
const STATE_SEED = "state"
const VAULTS_SEED = "vaults"

describe('state with default vaults', () => {
  const provider = anchor.AnchorProvider.env()
  const program = anchor.workspace.Protocol as Program<Protocol>

  const admin = Keypair.generate()
  const vaults = Keypair.generate()
  const vaults_size = program.account.vaults.size;
  const connection = program.provider.connection;

  anchor.setProvider(provider)

  // const [programAuthority, nonce] = PublicKey.findProgramAddressSync(
  //   [Buffer.from(SEED)],
  //   program.programId
  // )

  const [state_address, bump] = PublicKey.findProgramAddressSync(
    [Buffer.from(anchor.utils.bytes.utf8.encode(STATE_SEED))],
    program.programId
  )


  const [vaults_address, vaults_bump] = PublicKey.findProgramAddressSync(
    [Buffer.from(anchor.utils.bytes.utf8.encode(VAULTS_SEED))],
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


    // console.log(vaults_size + 8)
    const create_vaults_account_ix = SystemProgram.createAccount({
      fromPubkey: admin.publicKey,
      newAccountPubkey: vaults.publicKey,
      space: 14519,
      lamports: await provider.connection.getMinimumBalanceForRentExemption(
        14519
      ),
      programId: program.programId
    })

    tx.add(create_vaults_account_ix)

    const create_state_ix = await program.methods.createState(bump).accounts({
      admin: admin.publicKey,
      state: state_address,
      rent: SYSVAR_RENT_PUBKEY,
      systemProgram: SystemProgram.programId,
      vaults: vaults.publicKey
    }).instruction()

    tx.add(create_state_ix)

    // const extend_vaults_ix = await program.methods.extendVaults(vaults_bump).accounts({
    //   admin: admin.publicKey,
    //   state: state_address,
    //   rent: SYSVAR_RENT_PUBKEY,
    //   systemProgram: SystemProgram.programId,
    //   vaults: vaults_address
    // }).instruction()

    // tx.add(extend_vaults_ix)

    tx.recentBlockhash = blockhash
    tx.feePayer = admin.publicKey
    tx.partialSign(admin, vaults)

    const raw_tx = tx.serialize()
    const final_signature = await provider.connection.sendRawTransaction(raw_tx)

    await program.provider.connection.confirmTransaction({
      blockhash,
      lastValidBlockHeight,
      signature: final_signature
    })

    //const vaults_account = await program.account.vaults.fetch(vaults_address)
    const state_account = await program.account.state.fetch(state_address)


    // assert.ok(vaults_account)
    assert.equal(state_account.admin.toString(), admin.publicKey.toString())
    assert.equal(state_account.bump, bump)
  })
})

