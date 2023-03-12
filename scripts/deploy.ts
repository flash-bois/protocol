import * as anchor from '@coral-xyz/anchor'
import { Program, BN } from '@coral-xyz/anchor'
import { clusterApiUrl, PublicKey, sendAndConfirmTransaction, Signer } from '@solana/web3.js'

import { Protocol } from '../target/types/protocol'
import { admin, minter } from '../microSdk'
import { createDevnetEnvironment, IModifyCurveType, createStateWithVaults, IStateWithVaults } from '../tests/utils/utils'
import { createMint } from '@solana/spl-token'

anchor.setProvider(
  anchor.AnchorProvider.local(clusterApiUrl('devnet'), {
    skipPreflight: true
  })
)
const provider = anchor.getProvider()

const program = anchor.workspace.Protocol as Program<Protocol>

//@ts-ignore
const wallet: Signer = provider.wallet.payer

const connection = program.provider.connection

const main = async () => {
  await connection.requestAirdrop(admin.publicKey, 1000000000)

  const quote_mint = await createMint(connection, admin, minter.publicKey, null, 6)
  const base_mint1 = await createMint(connection, admin, minter.publicKey, null, 6)
  const base_mint2 = await createMint(connection, admin, minter.publicKey, null, 6)

  const state_with_vault = await createStateWithVaults({ admin, ix_only: false, program }) as IStateWithVaults

  const vault_txs = await createDevnetEnvironment({
    ix_only: true,
    ...state_with_vault,
    admin,
    minter: minter.publicKey,
    program,
    vaults_infos: [
      {
        base_mint: base_mint1,
        quote_mint,
        base_oracle: {
          oracle: new PublicKey('EhgAdTrgxi4ZoVZLQx1n93vULucPpiFi2BQtz9RJr1y6'),
          base: true,
          decimals: 6,
          skip_init: false,
          max_update_interval: 1
        },
        quote_oracle: {
          oracle: new PublicKey('5SSkXsEKQepHHAewytPVwdej4epN1nxgLVM84L4KXgy7'),
          base: false,
          decimals: 6,
          skip_init: false,
          max_update_interval: 1
        },
        lending: {
          fees: [
            {
              a: new BN(0),
              b: new BN(0),
              c: new BN(0),
              bound: new BN(10000),
              which: IModifyCurveType.Lend
            }
          ],
          initial_fee_time: 0,
          max_borrow: new BN(10_000_000_000),
          max_utilization: 800000
        },
        swapping: {
          fees: [
            {
              a: new BN(0),
              b: new BN(0),
              c: new BN(0),
              bound: new BN(10000),
              which: IModifyCurveType.SwapBuy
            },
            {
              a: new BN(0),
              b: new BN(0),
              c: new BN(0),
              bound: new BN(10000),
              which: IModifyCurveType.SwapSell
            }
          ],
          kept_fee: 100000,
          max_total_sold: new BN(10_000_000_000)
        },
        strategies: [
          {
            collateral_ratio: new BN(1000000),
            liquidation_threshold: new BN(1000000),
            lend: true,
            swap: true,
            trade: false
          }
        ]
      },
      {
        base_mint: base_mint2,
        quote_mint,
        base_oracle: {
          oracle: new PublicKey('A1WttWF7X3Rg6ZRpB2YQUFHCRh1kiXV8sKKLV3S9neJV'),
          base: true,
          decimals: 6,
          skip_init: false,
          max_update_interval: 1
        },
        quote_oracle: {
          oracle: new PublicKey('5SSkXsEKQepHHAewytPVwdej4epN1nxgLVM84L4KXgy7'),
          base: false,
          decimals: 6,
          skip_init: false,
          max_update_interval: 1
        },
        lending: {
          fees: [
            {
              a: new BN(0),
              b: new BN(0),
              c: new BN(0),
              bound: new BN(10000),
              which: IModifyCurveType.Lend
            }
          ],
          initial_fee_time: 0,
          max_borrow: new BN(1_000_000_000_000),
          max_utilization: 800000
        },
        swapping: {
          fees: [
            {
              a: new BN(0),
              b: new BN(0),
              c: new BN(0),
              bound: new BN(10000),
              which: IModifyCurveType.SwapBuy
            },
            {
              a: new BN(0),
              b: new BN(0),
              c: new BN(0),
              bound: new BN(10000),
              which: IModifyCurveType.SwapSell
            }
          ],
          kept_fee: 100000,
          max_total_sold: new BN(1_000_000_000_000)
        },
        strategies: [
          {
            collateral_ratio: new BN(1000000),
            liquidation_threshold: new BN(1000000),
            lend: false,
            swap: true,
            trade: false
          }
        ]
      }
    ]
  })

  const len = vault_txs.length
  let i = 0;

  while (i < len) {
    try {
      sendAndConfirmTransaction(provider.connection, vault_txs[i], [admin], { 'skipPreflight': true })
    } catch (err) {
      continue;
    }

    i++
  }
}

main()
