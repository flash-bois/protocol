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

//@ts-ignore
const wallet: Signer = provider.wallet.payer

const vaults_size = program.account.vaults.size
const connection = program.provider.connection

const [state_address, bump] = PublicKey.findProgramAddressSync(
  [Buffer.from(anchor.utils.bytes.utf8.encode(STATE_SEED))],
  program.programId
)

const main = async () => {
  const state = await program.account.state.fetch(state_address)

  const accountInfo = await connection.getAccountInfo(state.vaultsAcc)

  if (accountInfo === null) {
    throw new Error('Account not found')
  }

  const vaults = VaultsAccount.load(accountInfo.data)
  console.log(vaults.swap(0, 2n, true, false, 0))
}
main()
