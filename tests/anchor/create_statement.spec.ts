import * as anchor from '@coral-xyz/anchor'
import { Program } from '@coral-xyz/anchor'
import { Keypair, PublicKey, SystemProgram, SYSVAR_RENT_PUBKEY, Transaction } from '@solana/web3.js'
import { assert, use } from 'chai'
import { Protocol } from '../../target/types/protocol'
import { StateAccount, StatementAccount } from '../../pkg/protocol'

const STATEMENT_SEED = 'statement'

const sleep = (ms: number) => new Promise(resolve => setTimeout(resolve, ms))

describe('statement for user', () => {
  const provider = anchor.AnchorProvider.env()
  const program = anchor.workspace.Protocol as Program<Protocol>
  const connection = program.provider.connection

  const user = Keypair.generate()

  anchor.setProvider(provider)

  const [statement_address, bump] = PublicKey.findProgramAddressSync(
    [Buffer.from(anchor.utils.bytes.utf8.encode(STATEMENT_SEED)), user.publicKey.toBuffer()],
    program.programId
  )

  console.log(bump)

  it('Creates statement', async () => {
    let tx = new Transaction()

    const airdrop_signature = await connection.requestAirdrop(user.publicKey, 1000000000)
    const { blockhash, lastValidBlockHeight } = await connection.getLatestBlockhash()

    await program.provider.connection.confirmTransaction({
      signature: airdrop_signature,
      blockhash,
      lastValidBlockHeight
    })

    const create_statement_ix = await program.methods
      .createStatement()
      .accounts({
        payer: user.publicKey,
        rent: SYSVAR_RENT_PUBKEY,
        systemProgram: SystemProgram.programId,
        statement: statement_address
      })
      .instruction()

    tx.add(create_statement_ix)
    tx.recentBlockhash = blockhash
    tx.feePayer = user.publicKey

    tx.partialSign(user)

    const raw_tx = tx.serialize()
    const final_signature = await provider.connection.sendRawTransaction(raw_tx, {
      skipPreflight: true
    })

    await program.provider.connection.confirmTransaction({
      blockhash,
      lastValidBlockHeight,
      signature: final_signature
    })

    await sleep(5000)

    console.log(program.account.statement.size)
    const statement_account = await program.account.statement.fetch(statement_address)
    console.log(statement_account)
    // assert.equal(statement_account.owner.toString(), user.publicKey.toString())

    let account_info = (await connection.getAccountInfo(statement_address))?.data
    console.log(account_info?.toString('hex'))

    if (account_info) {
      const state = StatementAccount.load(account_info)
      console.log(state.get_bump())
    }
  })
})
