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
import { StateAccount, VaultsAccount } from './pkg/protocol'
import { Protocol } from './target/types/protocol'

export const minter = Keypair.fromSecretKey(
  Uint8Array.from(
    Buffer.from(
      '5e7ef659746de5b63e8a215f2c72cf4a293603cfb312545158e59907f269878c10bf87918327eb281d0e0e98725fce76ebb3a7bb530003d591115d49b5a8431d',
      'hex'
    )
  )
)
export const admin = Keypair.fromSecretKey(
  Uint8Array.from(
    Buffer.from(
      '49516d75930eff70fefa3721921a7e9738983b5194bf08d27a290948b0345df3fbc3033a77a9cf57240874269632afaa2d9faff98aa9919caa6ff52dad80cdf3',
      'hex'
    )
  )
)

// export const moderator = Keypair.fromSecretKey(
//   Uint8Array.from(
//     Buffer.from(
//       '57d2cc6607611114aede7d1b4be15980fc6c4728e3bd7a9b837a189bd35fe0cc7dec2daa8a768e5f79b0520fe6f4de747bcc902443a5507f798528d62bc142ab',
//       'hex'
//     )
//   )
// )

export const STATEMENT_SEED = 'statement'
export const STATE_SEED = 'state'
