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
import { STATE_SEED, admin, minter } from '../microSdk'
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

  const fetchedState = await program.account.state.fetch(state_address)
  const vaults = fetchedState.vaultsAcc

  const accountInfo = await connection.getAccountInfo(vaults)
  const vaultsAccount = VaultsAccount.load(accountInfo!.data)
  const index = vaultsAccount.vaults_len()

  console.log(`index: ${index}`)
  assert.equal(index, 1)

  const base = await createMint(connection, admin, minter.publicKey, null, 6)
  const quote =
    index === 0
      ? await createMint(connection, admin, minter.publicKey, null, 6)
      : new PublicKey(vaultsAccount.quote_token(0))
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
        .enableOracle(index, 6, true, false, 10)
        .accountsStrict({
          ...accounts,
          // priceFeed: new PublicKey('EhgAdTrgxi4ZoVZLQx1n93vULucPpiFi2BQtz9RJr1y6') // RAY
          priceFeed: new PublicKey('A1WttWF7X3Rg6ZRpB2YQUFHCRh1kiXV8sKKLV3S9neJV') // ORCA
        })
        .instruction(),
      await program.methods
        .enableOracle(index, 6, false, false, 10)
        .accountsStrict({
          ...accounts,
          priceFeed: new PublicKey('5SSkXsEKQepHHAewytPVwdej4epN1nxgLVM84L4KXgy7') // USDC
        })
        .instruction(),
      // await program.methods
      //   // .forceOverrideOracle(index, true, 200, 1, -2, 42)
      //   .forceOverrideOracle(index, true, 500, 4, -3, 42)
      //   .accountsStrict(accounts)
      //   .instruction(),
      // await program.methods
      //   .forceOverrideOracle(index, false, 1000, 2, -3, 42)
      //   .accountsStrict(accounts)
      //   .instruction(),
      await program.methods
        .enableLending(index, 800000, new BN(10000_000000), 0)
        .accounts(accounts)
        .signers([admin])
        .instruction(),
      await program.methods
        .enableSwapping(index, 100000, new BN(10000_000000))
        .accounts(accounts)
        .signers([admin])
        .instruction(),
      await program.methods
        .addStrategy(index, true, false, false, new BN(1000_000), new BN(1000_000))
        .accounts(accounts)
        .signers([admin])
        .instruction(),
      await program.methods
        .modifyFeeCurve(index, 2, true, new BN(1000000), new BN(0), new BN(0), new BN(100))
        .accounts(accounts)
        .signers([admin])
        .instruction(),
      await program.methods
        .modifyFeeCurve(index, 2, false, new BN(2000000), new BN(0), new BN(0), new BN(100))
        .accounts(accounts)
        .signers([admin])
        .instruction(),
      await program.methods
        .modifyFeeCurve(index, 1, true, new BN(1000000), new BN(0), new BN(0), new BN(1000))
        .accounts(accounts)
        .signers([admin])
        .instruction()
    ])
    .signers([admin, reserveBase, reserveQuote])
    .rpc({ skipPreflight: true })

  console.log(sig)
}

main()
