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
import { STATE_SEED, minter } from '../microSdk'

const provider = anchor.AnchorProvider.local(clusterApiUrl('devnet'), {
  skipPreflight: true
})

const program = anchor.workspace.Protocol as Program<Protocol>

const admin = Keypair.generate()
const vaults = Keypair.generate()
//@ts-ignore
const wallet: Signer = provider.wallet.payer

const vaults_size = program.account.vaults.size
const connection = program.provider.connection

anchor.setProvider(provider)

const [state_address, bump] = PublicKey.findProgramAddressSync(
  [Buffer.from(anchor.utils.bytes.utf8.encode(STATE_SEED))],
  program.programId
)

const main = async () => {
  const minter = Keypair.generate()

  const h = Buffer.from(minter.secretKey).toString('hex')
  console.log(h)
}

main()
