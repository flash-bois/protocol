import * as anchor from '@coral-xyz/anchor'
import { Provider, BN, Program, utils } from '@coral-xyz/anchor'
import { Keypair, PublicKey, SystemProgram, SYSVAR_RENT_PUBKEY, Transaction } from '@solana/web3.js'
import { Protocol } from '../../target/types/protocol'

const SEED = "DotWave"
const STATE_SEED = "state"

describe('state with default vaults', () => {
  it('creates state', async () => {
    const program = anchor.workspace.Protocol as Program<Protocol>


    const admin = Keypair.generate()
    const vaults = Keypair.generate()
    const vaults_size = program.account.vaults.size
    anchor.setProvider(anchor.AnchorProvider.local())

    const [programAuthority, nonce] = PublicKey.findProgramAddressSync(
      [Buffer.from(SEED)],
      program.programId
    )

    const [state_address, bump] = PublicKey.findProgramAddressSync(
      [Buffer.from(utils.bytes.utf8.encode(STATE_SEED))],
      program.programId
    )


    await program.provider.connection.requestAirdrop(admin.publicKey, 10000000000)
    await new Promise(f => setTimeout(f, 6000));



    // let pre_ix = SystemProgram.createAccount({
    //   fromPubkey: admin.publicKey,
    //   newAccountPubkey: vaults.publicKey,
    //   space: vaults_size,
    //   lamports: await program.provider.connection.getMinimumBalanceForRentExemption(
    //     vaults_size
    //   ),
    //   programId: program.programId
    // })

    let create_state_tx = await program.methods.createState(nonce).accounts({
      admin: admin.publicKey,
      state: state_address,
      programAuthority: programAuthority,
      rent: SYSVAR_RENT_PUBKEY,
      systemProgram: SystemProgram.programId,
      vaults: vaults.publicKey
    }).signers([admin]).rpc()

    // tx.add(create_state_tx)
    // let blockhash = (await provider.connection.getLatestBlockhash('recent')).blockhash;
    // tx.recentBlockhash = blockhash;
    // tx.feePayer = admin.publicKey

    // const signers: Keypair[] = [vaults, admin]
    // tx.partialSign(...signers)
    // const raw_tx = tx.serialize()



    // await provider.connection.sendRawTransaction(raw_tx)

    // const[userStatsPDA, _] = await PublicKey.findProgramAddress(
    //   [
    //     anchor.utils.bytes.utf8.encode('user-stats'),
    //     provider.wallet.publicKey.toBuffer(),
    //   ],
    //   program.programId
    // )
  })
})



