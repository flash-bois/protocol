import * as anchor from '@coral-xyz/anchor'
import { BN, Program, stateDiscriminator } from '@coral-xyz/anchor'
import {
  createAccount,
  createAssociatedTokenAccount,
  createMint,
  getAccount,
  mintTo,
  TOKEN_PROGRAM_ID
} from '@solana/spl-token'
import {
  Connection,
  Keypair,
  PublicKey,
  Signer,
  SystemProgram,
  SYSVAR_RENT_PUBKEY
} from '@solana/web3.js'
import { VaultsAccount } from '../../pkg/protocol'
import { Protocol } from '../../target/types/protocol'
import { STATE_SEED } from '../../microSdk'
import { Oracle } from '../../target/types/oracle'

export interface IProtocolCallable {
  program: Program<Protocol>
  admin: Keypair
}

export interface IOracleCallable {
  oracle_program: Program<Oracle>
  admin: Keypair
}

export interface IProtocolWithOracleCallable extends IProtocolCallable, IOracleCallable { }

export interface IStateWithVaults {
  state: PublicKey
  vaults: PublicKey
}

export interface IStrategyInfo {
  lend: boolean
  swap: boolean
  trade: boolean
  collateral_ratio: BN
  liquidation_threshold: BN
}

export interface ICreateStrategy extends IProtocolCallable, IStateWithVaults, IStrategyInfo {
  vault: number
}


export interface ISwappingInfo {
  kept_fee: number
  max_total_sold: BN
}

export interface IEnableSwapping extends IProtocolCallable, IStateWithVaults, ISwappingInfo {
  vault: number
}

export interface ILendingInfo {
  initial_fee_time: number
  max_utilization: number
  max_borrow: BN
}

export interface IEnableLending extends IProtocolCallable, IStateWithVaults, ILendingInfo {
  vault: number
}

export interface ICreateOracleInfo {
  price: BN
  conf: BN
  exp: number
}

export interface IEnableOracleInfo {
  base: boolean
  decimals: number
  skip_init: boolean
}

export interface ILocalOracleInfo extends ICreateOracleInfo, IEnableOracleInfo { }

export interface ICreateAndEnableOracle
  extends IProtocolWithOracleCallable,
  IStateWithVaults,
  ILocalOracleInfo {
  vault: number
}

export interface IAddVault extends IProtocolCallable, IStateWithVaults {
  minter: PublicKey
}

export interface IVaultInfo {
  quote_oracle?: ILocalOracleInfo
  base_oracle?: ILocalOracleInfo
  lending?: ILendingInfo
  swapping?: ISwappingInfo
  strategies?: [IStrategyInfo]
}
export interface ICreateTestEnvironment
  extends IProtocolWithOracleCallable {
  minter: PublicKey,
  vaults_infos: IVaultInfo[]
}

export type OracleKey = PublicKey

export interface IVaultAccounts {
  base: PublicKey
  quote: PublicKey
  reserveBase: PublicKey
  reserveQuote: PublicKey
  base_oracle?: OracleKey
  quote_oracle?: OracleKey
  remaining_accounts?: {
    isSigner: boolean;
    isWritable: boolean;
    pubkey: anchor.web3.PublicKey;
  }[]
}


export interface TestEnvironment extends IStateWithVaults {
  vaults_data: IVaultAccounts[]
}

export interface DotWaveAccounts {
  state: PublicKey
  vaults: PublicKey
  base: PublicKey
  quote: PublicKey
  reserveBase: PublicKey
  reserveQuote: PublicKey
  otherBases?: PublicKey[]
  otherReserveBases?: PublicKey[]
  otherReserveQuotes?: PublicKey[]
}

export interface AdminAccounts {
  state: PublicKey
  vaults: PublicKey
  admin: PublicKey
}

export interface Loadable<T> {
  load(Buffer): T
}

export async function createBasicVault(
  program: Program<Protocol>,
  admin: Keypair,
  minter: Keypair,
  existing?: DotWaveAccounts,
  index = 0
): Promise<DotWaveAccounts> {
  const result = await initAccounts(program, admin, minter, existing)

  const accounts = {
    state: result.state,
    vaults: result.vaults,
    admin: admin.publicKey
  }
  const i = index
  await enableOracles(program, i, accounts, admin)
  await program.methods
    .enableLending(i, 800000, new BN(10000_000000), 0)
    .accounts(accounts)
    .signers([admin])
    .postInstructions([
      await program.methods
        .enableSwapping(i, 100000, new BN(10000_000000))
        .accounts(accounts)
        .signers([admin])
        .instruction(),
      await program.methods
        .addStrategy(i, true, false, new BN(1000000), new BN(1000000))
        .accounts(accounts)
        .signers([admin])
        .instruction()
    ])
    .rpc({ skipPreflight: true })
  return result
}

export async function mintTokensForUser(
  connection: Connection,
  minter: Keypair,
  user: Keypair,
  base: PublicKey,
  quote: PublicKey,
  mintAmount: number | bigint = 1e9
) {
  // const [accountBase, accountQuote] = await Promise.all([
  //   createAccount(connection, user, base, user.publicKey),
  //   createAccount(connection, user, quote, user.publicKey)
  // ])

  // await Promise.all([
  //   mintTo(connection, user, base, accountBase, minter, mintAmount),
  //   mintTo(connection, user, quote, accountQuote, minter, mintAmount)
  // ])

  const accountBase = await createAssociatedTokenAccount(connection, user, base, user.publicKey)
  const accountQuote = await createAssociatedTokenAccount(connection, user, quote, user.publicKey)

  await Promise.all([
    mintTo(connection, user, base, accountBase, minter, 1e6),
    mintTo(connection, user, quote, accountQuote, minter, 1e6)
  ])

  return {
    accountBase,
    accountQuote
  }
}

export async function createAccounts(
  program: Program<Protocol>,
  admin: Keypair
): Promise<{ state: PublicKey; vaults: PublicKey }> {
  const vaults = Keypair.generate()

  const [state, bump] = PublicKey.findProgramAddressSync(
    [Buffer.from(anchor.utils.bytes.utf8.encode(STATE_SEED))],
    program.programId
  )

  await program.methods
    .createState()
    .accounts({
      admin: admin.publicKey,
      state,
      rent: SYSVAR_RENT_PUBKEY,
      systemProgram: SystemProgram.programId,
      vaults: vaults.publicKey
    })
    .preInstructions([
      SystemProgram.createAccount({
        fromPubkey: admin.publicKey,
        newAccountPubkey: vaults.publicKey,
        space: VaultsAccount.size(),
        lamports: await program.provider.connection.getMinimumBalanceForRentExemption(
          VaultsAccount.size()
        ),
        programId: program.programId
      })
    ])
    .signers([admin, vaults])
    .rpc({ skipPreflight: true })

  return {
    state,
    vaults: vaults.publicKey
  }
}

export async function initAccounts(
  program: Program<Protocol>,
  admin: Keypair,
  minter: Keypair,
  existing?: DotWaveAccounts
): Promise<DotWaveAccounts> {
  const connection = program.provider.connection
  let state
  let vaults

  if (!existing) {
    const vaultsKeypair = Keypair.generate()
    const [stateAddress, bump] = PublicKey.findProgramAddressSync(
      [Buffer.from(anchor.utils.bytes.utf8.encode(STATE_SEED))],
      program.programId
    )
    state = stateAddress
    vaults = vaultsKeypair.publicKey

    await program.methods
      .createState()
      .accounts({
        admin: admin.publicKey,
        state,
        rent: SYSVAR_RENT_PUBKEY,
        systemProgram: SystemProgram.programId,
        vaults: vaultsKeypair.publicKey
      })
      .preInstructions([
        SystemProgram.createAccount({
          fromPubkey: admin.publicKey,
          newAccountPubkey: vaultsKeypair.publicKey,
          space: VaultsAccount.size(),
          lamports: await connection.getMinimumBalanceForRentExemption(VaultsAccount.size()),
          programId: program.programId
        })
      ])
      .signers([admin, vaultsKeypair])
      .rpc({ skipPreflight: true })
  } else {
    state = existing.state
    vaults = existing.vaults
  }

  const quote = existing?.quote ?? (await createMint(connection, admin, minter.publicKey, null, 6))

  const base = await createMint(connection, admin, minter.publicKey, null, 6)
  const reserveBase = Keypair.generate()
  const reserveQuote = Keypair.generate()

  await program.methods
    .initVault()
    .accounts({
      state,
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

  return {
    state,
    vaults,
    base,
    quote,
    reserveBase: reserveBase.publicKey,
    reserveQuote: reserveQuote.publicKey
  }
}

export async function waitFor(connection: Connection, sig: string) {
  const { blockhash, lastValidBlockHeight } = await connection.getLatestBlockhash()
  await connection.confirmTransaction(
    {
      signature: sig,
      blockhash,
      lastValidBlockHeight
    },
    'singleGossip'
  )
}

export async function sleep(ms: number) {
  return new Promise(resolve => setTimeout(resolve, ms))
}

export async function enableOracles(
  program: Program<Protocol>,
  index: number,
  accounts: AdminAccounts,
  admin: Signer
) {
  const priceFeed = Keypair.generate().publicKey
  const { state, vaults, admin: adminKey } = accounts
  await program.methods
    .enableOracle(index, 6, true, true)
    .accountsStrict({
      ...accounts,
      priceFeed
    })
    .postInstructions([
      await program.methods
        .enableOracle(index, 6, false, true)
        .accountsStrict({
          ...accounts,
          priceFeed
        })
        .instruction(),
      await program.methods
        .forceOverrideOracle(index, true, 200, 1, -2, 42)
        .accountsStrict(accounts)
        .instruction(),
      await program.methods
        .forceOverrideOracle(index, false, 1000, 2, -3, 42)
        .accountsStrict(accounts)
        .instruction()
    ])
    .signers([admin])
    .rpc({ skipPreflight: true })
}

export async function enableSwapping({ program, admin, ...params }: IEnableSwapping) {
  const { vault, kept_fee, max_total_sold, ...common_accounts } = params

  const sig = await program.methods
    .enableSwapping(vault, kept_fee, max_total_sold)
    .accounts({
      admin: admin.publicKey,
      ...common_accounts
    })
    .signers([admin])
    .rpc({ skipPreflight: true })

  await waitFor(program.provider.connection, sig)
}

export async function enableLending({ program, admin, ...params }: IEnableLending) {
  const { vault, initial_fee_time, max_borrow, max_utilization, ...common_accounts } = params

  const sig = await program.methods
    .enableLending(vault, max_utilization, max_borrow, initial_fee_time)
    .accounts({
      admin: admin.publicKey,
      ...common_accounts
    })
    .signers([admin])
    .rpc({ skipPreflight: true })

  await waitFor(program.provider.connection, sig)
}

export async function createStrategy({
  vault,
  program,
  admin,

  ...params
}: ICreateStrategy) {
  const {
    lend,
    swap,
    trade,
    collateral_ratio,
    liquidation_threshold,

    ...common_accounts
  } = params

  const sig = await program.methods
    .addStrategy(vault, lend, swap, collateral_ratio, liquidation_threshold)
    .accounts({ admin: admin.publicKey, ...common_accounts })
    .signers([admin])
    .rpc({ skipPreflight: true })

  await waitFor(program.provider.connection, sig)
}

export async function createAndEnableOracle({
  oracle_program,
  program,
  admin,

  ...params
}: ICreateAndEnableOracle): Promise<OracleKey> {
  const {
    vault,
    base,
    conf,
    exp,
    price,
    decimals,
    skip_init,

    ...common_accounts
  } = params

  const oracle = Keypair.generate()
  const oracle_connection = oracle_program.provider.connection
  const connection = program.provider.connection

  const create_sig = await oracle_program.methods
    .set(price, exp, conf)
    .preInstructions([
      SystemProgram.createAccount({
        fromPubkey: admin.publicKey,
        newAccountPubkey: oracle.publicKey,
        space: 3312,
        lamports: await oracle_connection.getMinimumBalanceForRentExemption(3312),
        programId: oracle_program.programId
      })
    ])
    .accounts({ price: oracle.publicKey, signer: admin.publicKey })
    .signers([oracle, admin])
    .rpc({ skipPreflight: true })

  await waitFor(oracle_connection, create_sig)

  const enable_sig = await program.methods
    .enableOracle(vault, decimals, base, skip_init)
    .accounts({
      priceFeed: oracle.publicKey,
      admin: admin.publicKey,
      ...common_accounts
    })
    .signers([admin])
    .rpc({ skipPreflight: true })

  await waitFor(connection, enable_sig)

  return oracle.publicKey
}

export async function createStateWithVaults(params: IProtocolCallable): Promise<IStateWithVaults> {
  const { admin, program } = params

  const vaults = Keypair.generate()
  const connection = program.provider.connection

  const [state, bump] = PublicKey.findProgramAddressSync(
    [Buffer.from(anchor.utils.bytes.utf8.encode(STATE_SEED))],
    program.programId
  )

  const sig = await program.methods
    .createState()
    .accounts({
      admin: admin.publicKey,
      state,
      rent: SYSVAR_RENT_PUBKEY,
      systemProgram: SystemProgram.programId,
      vaults: vaults.publicKey
    })
    .preInstructions([
      SystemProgram.createAccount({
        fromPubkey: admin.publicKey,
        newAccountPubkey: vaults.publicKey,
        space: VaultsAccount.size(),
        lamports: await connection.getMinimumBalanceForRentExemption(VaultsAccount.size()),
        programId: program.programId
      })
    ])
    .signers([admin, vaults])
    .rpc({ skipPreflight: true })

  await waitFor(connection, sig)

  return { vaults: vaults.publicKey, state }
}

export async function createTestEnvironment({ vaults_infos, ...params }: ICreateTestEnvironment): Promise<TestEnvironment> {
  const state_with_vault = await createStateWithVaults(params)
  const vaults_data: IVaultAccounts[] = []

  for (const id of vaults_infos.keys()) {
    const {
      base_oracle,
      quote_oracle,
      lending,
      swapping,
      strategies
    }: IVaultInfo = vaults_infos[id]

    const vault_accounts = await addVault({ ...state_with_vault, ...params })

    const base_oracle_key = base_oracle ? await createAndEnableOracle({ vault: id, ...base_oracle, ...params, ...state_with_vault, }) : undefined
    const quote_oracle_key = quote_oracle ? await createAndEnableOracle({ vault: id, ...quote_oracle, ...params, ...state_with_vault, }) : undefined

    if (lending != undefined) {
      await enableLending({ vault: id, ...lending, ...params, ...state_with_vault })
    }

    if (swapping != undefined) {
      await enableSwapping({ vault: id, ...swapping, ...params, ...state_with_vault, })
    }

    if (strategies != undefined) {
      for (const id of strategies.keys()) {
        const strategy = strategies[id]
        await createStrategy({ vault: id, ...strategy, ...params, ...state_with_vault, })
      }
    }

    let remaining_accounts: { isSigner: false, isWritable: false, pubkey: PublicKey }[] = []

    if (base_oracle) {
      remaining_accounts.push({ isSigner: false, isWritable: false, pubkey: base_oracle_key! })
    }

    if (quote_oracle) {
      remaining_accounts.push({ isSigner: false, isWritable: false, pubkey: quote_oracle_key! })
    }


    vaults_data.push(
      {
        ...vault_accounts, base_oracle: base_oracle_key, quote_oracle: quote_oracle_key, remaining_accounts
      }
    )
  }



  return { ...state_with_vault, vaults_data }
}

export async function addVault({
  admin,
  minter,
  program,
  ...common_accounts
}: IAddVault): Promise<IVaultAccounts> {
  const connection = program.provider.connection
  const base = await createMint(connection, admin, minter, null, 6)
  const quote = await createMint(connection, admin, minter, null, 6)
  const reserveBase = Keypair.generate()
  const reserveQuote = Keypair.generate()

  const sig = await program.methods
    .initVault()
    .accounts({
      ...common_accounts,
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

  await waitFor(connection, sig)

  return { base, quote, reserveBase: reserveBase.publicKey, reserveQuote: reserveQuote.publicKey }
}

export async function tryFetch(connection: Connection, address: PublicKey): Promise<Buffer> {
  let data = (await connection.getAccountInfo(address))?.data
  if (!data) throw 'could not fetch account info'
  return data
}
