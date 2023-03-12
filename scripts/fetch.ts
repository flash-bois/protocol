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
  // console.log(`vaults: ${vaults.vaults_len()}`)
  // console.log(`strategies : ${vaults.count_strategies(0)}`)
  // const stinfo = vaults.strategy_info(0, 0)
  // console.log(`strategy : ${stinfo.balance_base}, ${stinfo.balance_quote}`)

  console.log(`quote: ${new PublicKey(vaults.quote_token(0)).toString()}`)
  console.log(`base: ${new PublicKey(vaults.base_token(0)).toString()}`)

  for (let i = 0; i < vaults.vaults_len(); i++) {
    console.log(
      `vault ${i} tokens: ${vaults.base_token(i).toString()},  ${vaults.quote_token(i).toString()}`
    )
    console.log(
      `vault ${i} orales: ${vaults.oracle_base(i).toString()}, ${vaults.oracle_quote(i).toString()}`
    )
  }
}
main()
