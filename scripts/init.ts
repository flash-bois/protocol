import * as anchor from '@coral-xyz/anchor'
import { Program } from '@coral-xyz/anchor'
import {
  clusterApiUrl,
  Keypair,
  PublicKey,
  Signer,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
  Transaction
} from '@solana/web3.js'
import { assert } from 'chai'
import { StateAccount, VaultsAccount } from '../pkg/protocol'
import { Protocol } from '../target/types/protocol'
import { STATE_SEED, admin } from '../microSdk'

anchor.setProvider(
  anchor.AnchorProvider.local(clusterApiUrl('devnet'), {
    skipPreflight: true
  })
)
const provider = anchor.getProvider()

const program = anchor.workspace.Protocol as Program<Protocol>

const vaults = Keypair.generate()
//@ts-ignore
const wallet: Signer = provider.wallet.payer

const connection = program.provider.connection

const [state_address, bump] = PublicKey.findProgramAddressSync(
  [Buffer.from(anchor.utils.bytes.utf8.encode(STATE_SEED))],
  program.programId
)

const main = async () => {
  const sig = await program.methods
    .createState()
    .accounts({
      admin: admin.publicKey,
      state: state_address,
      rent: SYSVAR_RENT_PUBKEY,
      systemProgram: SystemProgram.programId,
      vaults: vaults.publicKey
    })
    .preInstructions([
      SystemProgram.createAccount({
        fromPubkey: admin.publicKey,
        newAccountPubkey: vaults.publicKey,
        space: VaultsAccount.size(),
        lamports: await connection.getMinimumBalanceForRentExemption(VaultsAccount.size()),
        programId: program.programId
      })
    ])
    .signers([admin, vaults])
    .rpc({ skipPreflight: true })

  console.log('Signature:', sig)
}

main()
