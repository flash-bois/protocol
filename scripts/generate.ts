import * as anchor from '@coral-xyz/anchor'
import { BN, Program } from '@coral-xyz/anchor'
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
import { STATE_SEED, admin, vaults, minter } from '../microSdk'
import { createMint, TOKEN_PROGRAM_ID } from '@solana/spl-token'

const program = anchor.workspace.Protocol as Program<Protocol>

const [state_address, bump] = PublicKey.findProgramAddressSync(
  [Buffer.from(anchor.utils.bytes.utf8.encode(STATE_SEED))],
  program.programId
)

console.log(
  JSON.stringify({
    stateAddress: state_address,
    vaultsAddress: vaults,
    programId: program.programId,
    stateSeed: STATE_SEED,
    statementSeed: 'statement',
    minterBuffer:
      '5e7ef659746de5b63e8a215f2c72cf4a293603cfb312545158e59907f269878c10bf87918327eb281d0e0e98725fce76ebb3a7bb530003d591115d49b5a8431d'
  })
)
