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
  console.log(connection)

  const index = 0

  const base = await createMint(connection, admin, minter.publicKey, null, 6)
  const quote = await createMint(connection, admin, minter.publicKey, null, 6)
  const reserveBase = Keypair.generate()
  const reserveQuote = Keypair.generate()

  console.log(`tokens: ${base.toString()}, ${quote.toString()}`)
  console.log(`reserves: ${reserveBase.publicKey.toString()}, ${reserveQuote.publicKey.toString()}`)

  const accounts = {
    state: state_address,
    vaults,
    admin: admin.publicKey
  }

  const sig = await program.methods
    .initVault()
    .accounts({
      state: state_address,
      vaults,
      base,
      quote,
      reserveBase: reserveBase.publicKey,
      reserveQuote: reserveQuote.publicKey,
      admin: admin.publicKey,
      tokenProgram: TOKEN_PROGRAM_ID,
      systemProgram: SystemProgram.programId
    })
    .postInstructions([
      await program.methods
        .enableOracle(index, 6, true, true)
        .accountsStrict({
          ...accounts,
          priceFeed: new PublicKey('H6ARHf6YXhGYeQfUzQNGk6rDNnLBQKrenN712K4AQJEG')
        })
        .instruction(),
      await program.methods
        .enableOracle(index, 6, false, true)
        .accountsStrict({
          ...accounts,
          priceFeed: new PublicKey('Gnt27xtC473ZT2Mw5u8wZ68Z3gULkSTb5DuxJy7eJotD')
        })
        .instruction(),
      await program.methods
        .forceOverrideOracle(0, true, 200, 1, -2, 42)
        .accountsStrict(accounts)
        .instruction(),
      await program.methods
        .forceOverrideOracle(0, false, 1000, 2, -3, 42)
        .accountsStrict(accounts)
        .instruction(),
      await program.methods
        .enableLending(index, 800000, new BN(10000_000000))
        .accounts(accounts)
        .signers([admin])
        .instruction(),
      await program.methods
        .enableSwapping(index, 100000, new BN(10000_000000))
        .accounts(accounts)
        .signers([admin])
        .instruction(),
      await program.methods
        .addStrategy(index, true, false)
        .accounts(accounts)
        .signers([admin])
        .instruction()
    ])
    .signers([admin, reserveBase, reserveQuote])
    .rpc({ skipPreflight: true })

  console.log(sig)
}

main()
