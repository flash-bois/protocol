import * as anchor from '@project-serum/anchor'
import { Provider, BN, Program, utils } from '@project-serum/anchor'
import { Keypair, PublicKey, SystemProgram, SYSVAR_RENT_PUBKEY, Transaction } from '@solana/web3.js'
import { Protocol } from '../../target/types/protocol'

const SEED = "DotWave"
const STATE_SEED = "state"

describe('state with default vaults', () => {
  const provider = anchor.AnchorProvider.env()
  const program = anchor.workspace.protocol as Program<Protocol>

  const admin = Keypair.generate()
  const vaults = Keypair.generate()
  const vaults_size = program.account.vaults.size;

  anchor.setProvider(provider)

  const [programAuthority, nonce] = PublicKey.findProgramAddressSync(
    [Buffer.from(SEED)],
    program.programId
  )

  const [state_address, bump] = PublicKey.findProgramAddressSync(
    [Buffer.from(utils.bytes.utf8.encode(STATE_SEED))],
    program.programId
  )

  it('Creates states!', async () => {
    let tx = new Transaction();

    let create_state_account_tx = SystemProgram.createAccount({
      fromPubkey: admin.publicKey,
      newAccountPubkey: vaults.publicKey,
      space: vaults_size,
      lamports: await provider.connection.getMinimumBalanceForRentExemption(
        vaults_size
      ),
      programId: program.programId
    })

    tx.add(create_state_account_tx)

    let create_state_tx = await program.methods.createState(nonce).accounts({
      admin: admin.publicKey,
      state: state_address,
      programAuthority: programAuthority,
      rent: SYSVAR_RENT_PUBKEY,
      systemProgram: SystemProgram.programId,
      vaults: vaults.publicKey
    }).transaction()

    tx.add(create_state_tx)

    const signers: Keypair[] = [vaults, admin]
    tx.partialSign(...signers)
    const raw_tx = tx.serialize()

    await provider.connection.sendRawTransaction(raw_tx)

    // const[userStatsPDA, _] = await PublicKey.findProgramAddress(
    //   [
    //     anchor.utils.bytes.utf8.encode('user-stats'),
    //     provider.wallet.publicKey.toBuffer(),
    //   ],
    //   program.programId
    // )
  })
})

