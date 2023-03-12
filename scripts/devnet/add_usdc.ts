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
import { StateAccount, VaultsAccount } from '../../pkg/protocol'
import { Protocol } from '../../target/types/protocol'
import { STATE_SEED, admin, minter, DEVNET_ORACLES } from '../../microSdk'
import { createMint, TOKEN_PROGRAM_ID } from '@solana/spl-token'

anchor.setProvider(
  anchor.AnchorProvider.local(clusterApiUrl('devnet'), {
    skipPreflight: true
  })
)
const provider = anchor.getProvider()

const program = anchor.workspace.Protocol as Program<Protocol>

const connection = program.provider.connection

const [stateAddress, bump] = PublicKey.findProgramAddressSync(
  [Buffer.from(anchor.utils.bytes.utf8.encode(STATE_SEED))],
  program.programId
)

const main = async () => {
  console.log(connection)

  const fetchedState = await program.account.state.fetch(stateAddress)
  const vaults = fetchedState.vaultsAcc

  const accountInfo = await connection.getAccountInfo(vaults)
  const vaultsAccount = VaultsAccount.load(accountInfo!.data)
  const index = vaultsAccount.vaults_len()

  console.log(`index: ${index}`)
  assert.equal(index, 3)

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
    state: stateAddress,
    vaults,
    admin: admin.publicKey
  }

  const sigv = await program.methods
    .initVault()
    .accounts({
      state: stateAddress,
      vaults,
      base,
      quote,
      reserveBase: reserveBase.publicKey,
      reserveQuote: reserveQuote.publicKey,
      admin: admin.publicKey,
      tokenProgram: TOKEN_PROGRAM_ID,
      systemProgram: SystemProgram.programId
    })
    .signers([admin, reserveBase, reserveQuote])
    .rpc({ skipPreflight: true })

  console.log('vault', sigv)

  const sig = await program.methods
    .enableOracle(index, 6, true, false, 1)
    .accountsStrict({
      ...accounts,
      priceFeed: DEVNET_ORACLES.USDC
    })
    .postInstructions([
      await program.methods
        .enableOracle(index, 6, false, false, 1)
        .accountsStrict({
          ...accounts,
          priceFeed: DEVNET_ORACLES.USDT
        })
        .instruction(),
      await program.methods
        .enableLending(index, 900000, new BN(10_000_000000000), 600) ///// LENDING
        .accounts(accounts)
        .signers([admin])
        .instruction(),
      await program.methods
        .modifyFeeCurve(index, 1, true, new BN(50000), new BN(0), new BN(0), new BN(10))
        .accounts(accounts)
        .signers([admin])
        .instruction(),
      await program.methods
        .modifyFeeCurve(index, 1, true, new BN(850000), new BN(0), new BN(125000), new BN(10))
        .accounts(accounts)
        .signers([admin])
        .instruction(),
      await program.methods
        .modifyFeeCurve(index, 1, true, new BN(1000000), new BN(0), new BN(100010), new BN(100))
        .accounts(accounts)
        .signers([admin])
        .instruction(),
      await program.methods
        .enableSwapping(index, 100000, new BN(10_000_000_000)) ///// SWAPPING
        .accounts(accounts)
        .signers([admin])
        .instruction(),
      await program.methods
        .modifyFeeCurve(index, 2, true, new BN(10000), new BN(0), new BN(0), new BN(100))
        .accounts(accounts)
        .signers([admin])
        .instruction(),
      await program.methods
        .modifyFeeCurve(index, 2, true, new BN(300000), new BN(0), new BN(1000), new BN(1000))
        .accounts(accounts)
        .signers([admin])
        .instruction(),
      await program.methods
        .modifyFeeCurve(index, 2, false, new BN(10000), new BN(0), new BN(0), new BN(100))
        .accounts(accounts)
        .signers([admin])
        .instruction(),
      await program.methods
        .modifyFeeCurve(index, 2, false, new BN(300000), new BN(0), new BN(1000), new BN(1000))
        .accounts(accounts)
        .signers([admin])
        .instruction(),
      // await program.methods
      //   .enableTrading(index, 10000, 5000000, 600000, 800000) ///// TRADING
      //   .accounts(accounts)
      //   .signers([admin])
      //   .instruction(),
      await program.methods
        .addStrategy(index, true, true, false, new BN(800_000), new BN(850_000)) ///// STRATEGIES
        .accounts(accounts)
        .signers([admin])
        .instruction()
      // await program.methods
      //   .addStrategy(index, true, true, true, new BN(600_000), new BN(700_000))
      //   .accounts(accounts)
      //   .signers([admin])
      //   .instruction()
    ])
    .signers([admin])
    .rpc({ skipPreflight: true })

  console.log(sig)
}

main()
