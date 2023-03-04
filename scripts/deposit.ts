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
  const user = Keypair.generate()
  connection.requestAirdrop(user.publicKey, 1e7)

  const state = await program.account.state.fetch(state_address)

  const accountInfo = await connection.getAccountInfo(state.vaultsAcc)

  if (accountInfo === null) throw new Error('Account not found')

  const vaults = VaultsAccount.load(accountInfo.data)
  const mint = new PublicKey(vaults.base_token(0))

  program.methods.deposit(0, 0, 1e6, true)
}
main()
