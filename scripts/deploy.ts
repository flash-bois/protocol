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
          oracle: new PublicKey('EhgAdTrgxi4ZoVZLQx1n93vULucPpiFi2BQtz9RJr1y6'), // SOL
          base: true,
          decimals: 9,
          skip_init: false,
          max_update_interval: 1
        },
        quote_oracle: {
          oracle: new PublicKey('3vxLXJqLqF3JG5TCbYycbKWRBbCJQLxQmBGCkyqEEefL'), // USDT
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
              c: new BN(10), //0.001%/h
              bound: new BN(50000),
              which: IModifyCurveType.Lend
            },
            {
              a: new BN(0),
              b: new BN(125000),
              c: new BN(10),
              bound: new BN(850000),
              which: IModifyCurveType.Lend
            },
            {
              a: new BN(0),
              b: new BN(0),
              c: new BN(100010),
              bound: new BN(1000000),
              which: IModifyCurveType.Lend
            }
          ],
          initial_fee_time: 600,
          max_borrow: new BN(10_000_000000000),
          max_utilization: 900000
        },
        swapping: {
          fees: [
            {
              a: new BN(0),
              b: new BN(0),
              c: new BN(1000),
              bound: new BN(10000),
              which: IModifyCurveType.SwapBuy
            },
            {
              a: new BN(0),
              b: new BN(10000),
              c: new BN(10000),
              bound: new BN(300000),
              which: IModifyCurveType.SwapBuy
            },
            {
              a: new BN(0),
              b: new BN(0),
              c: new BN(1000),
              bound: new BN(10000),
              which: IModifyCurveType.SwapSell
            },
            {
              a: new BN(0),
              b: new BN(10000),
              c: new BN(10000),
              bound: new BN(300000),
              which: IModifyCurveType.SwapSell
            }
          ],
          kept_fee: 100000,
          max_total_sold: new BN(10_000_000_000)
        },
        strategies: [
          {
            collateral_ratio: new BN(800000),
            liquidation_threshold: new BN(85000000),
            lend: true,
            swap: false,
            trade: false
          },
          {
            collateral_ratio: new BN(600000),
            liquidation_threshold: new BN(65000000),
            lend: true,
            swap: true,
            trade: false
          },
          {
            collateral_ratio: new BN(400000),
            liquidation_threshold: new BN(50000000),
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
          oracle: new PublicKey('JBu1AL4obBcCMqKBBxhpWCNUt136ijcuMZLFvTP7iWdB'), // ETH
          base: true,
          decimals: 9,
          skip_init: false,
          max_update_interval: 1
        },
        quote_oracle: {
          oracle: new PublicKey('3vxLXJqLqF3JG5TCbYycbKWRBbCJQLxQmBGCkyqEEefL'), // USDT
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
              c: new BN(10), //0.001%/h
              bound: new BN(50000),
              which: IModifyCurveType.Lend
            },
            {
              a: new BN(0),
              b: new BN(125000),
              c: new BN(10),
              bound: new BN(850000),
              which: IModifyCurveType.Lend
            },
            {
              a: new BN(0),
              b: new BN(0),
              c: new BN(100010),
              bound: new BN(1000000),
              which: IModifyCurveType.Lend
            }
          ],
          initial_fee_time: 600,
          max_borrow: new BN(10_000_000000000),
          max_utilization: 900000
        },
        swapping: {
          fees: [
            {
              a: new BN(0),
              b: new BN(0),
              c: new BN(1000),
              bound: new BN(10000),
              which: IModifyCurveType.SwapBuy
            },
            {
              a: new BN(0),
              b: new BN(10000),
              c: new BN(10000),
              bound: new BN(300000),
              which: IModifyCurveType.SwapBuy
            },
            {
              a: new BN(0),
              b: new BN(0),
              c: new BN(1000),
              bound: new BN(10000),
              which: IModifyCurveType.SwapSell
            },
            {
              a: new BN(0),
              b: new BN(10000),
              c: new BN(10000),
              bound: new BN(300000),
              which: IModifyCurveType.SwapSell
            }
          ],
          kept_fee: 100000,
          max_total_sold: new BN(10_000_000_000)
        },
        strategies: [
          {
            collateral_ratio: new BN(800000),
            liquidation_threshold: new BN(85000000),
            lend: true,
            swap: false,
            trade: false
          },
          {
            collateral_ratio: new BN(600000),
            liquidation_threshold: new BN(65000000),
            lend: true,
            swap: true,
            trade: false
          },
          {
            collateral_ratio: new BN(400000),
            liquidation_threshold: new BN(50000000),
            lend: true,
            swap: true,
            trade: false
          }
        ]
      },
      {
        quote_mint,
        base_oracle: {
          oracle: new PublicKey('E4v1BBgoso9s64TQvmyownAVJbhbEPGyzA3qn4n46qj9'), // MSOL
          base: true,
          decimals: 9,
          skip_init: false,
          max_update_interval: 1
        },
        quote_oracle: {
          oracle: new PublicKey('3vxLXJqLqF3JG5TCbYycbKWRBbCJQLxQmBGCkyqEEefL'), // USDT
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
              c: new BN(10), //0.001%/h
              bound: new BN(50000),
              which: IModifyCurveType.Lend
            },
            {
              a: new BN(0),
              b: new BN(125000),
              c: new BN(10),
              bound: new BN(850000),
              which: IModifyCurveType.Lend
            },
            {
              a: new BN(0),
              b: new BN(0),
              c: new BN(100010),
              bound: new BN(1000000),
              which: IModifyCurveType.Lend
            }
          ],
          initial_fee_time: 600,
          max_borrow: new BN(10_000_000000000),
          max_utilization: 900000
        },
        swapping: {
          fees: [
            {
              a: new BN(0),
              b: new BN(0),
              c: new BN(1000),
              bound: new BN(10000),
              which: IModifyCurveType.SwapBuy
            },
            {
              a: new BN(0),
              b: new BN(10000),
              c: new BN(10000),
              bound: new BN(300000),
              which: IModifyCurveType.SwapBuy
            },
            {
              a: new BN(0),
              b: new BN(0),
              c: new BN(1000),
              bound: new BN(10000),
              which: IModifyCurveType.SwapSell
            },
            {
              a: new BN(0),
              b: new BN(10000),
              c: new BN(10000),
              bound: new BN(300000),
              which: IModifyCurveType.SwapSell
            }
          ],
          kept_fee: 100000,
          max_total_sold: new BN(10_000_000_000)
        },
        strategies: [
          {
            collateral_ratio: new BN(800000),
            liquidation_threshold: new BN(85000000),
            lend: true,
            swap: false,
            trade: false
          },
          {
            collateral_ratio: new BN(600000),
            liquidation_threshold: new BN(65000000),
            lend: true,
            swap: true,
            trade: false
          },
          {
            collateral_ratio: new BN(400000),
            liquidation_threshold: new BN(50000000),
            lend: true,
            swap: true,
            trade: false
          }
        ]
      },
      {
        quote_mint,
        base_oracle: {
          oracle: new PublicKey('Gnt27xtC473ZT2Mw5u8wZ68Z3gULkSTb5DuxJy7eJotD'), // USDC
          base: true,
          decimals: 9,
          skip_init: false,
          max_update_interval: 1
        },
        quote_oracle: {
          oracle: new PublicKey('3vxLXJqLqF3JG5TCbYycbKWRBbCJQLxQmBGCkyqEEefL'), // USDT
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
              c: new BN(10), //0.001%/h
              bound: new BN(50000),
              which: IModifyCurveType.Lend
            },
            {
              a: new BN(0),
              b: new BN(125000),
              c: new BN(10),
              bound: new BN(850000),
              which: IModifyCurveType.Lend
            },
            {
              a: new BN(0),
              b: new BN(0),
              c: new BN(100010),
              bound: new BN(1000000),
              which: IModifyCurveType.Lend
            }
          ],
          initial_fee_time: 600,
          max_borrow: new BN(10_000_000000000),
          max_utilization: 900000
        },
        swapping: {
          fees: [
            {
              a: new BN(0),
              b: new BN(0),
              c: new BN(1000),
              bound: new BN(10000),
              which: IModifyCurveType.SwapBuy
            },
            {
              a: new BN(0),
              b: new BN(10000),
              c: new BN(10000),
              bound: new BN(300000),
              which: IModifyCurveType.SwapBuy
            },
            {
              a: new BN(0),
              b: new BN(0),
              c: new BN(1000),
              bound: new BN(10000),
              which: IModifyCurveType.SwapSell
            },
            {
              a: new BN(0),
              b: new BN(10000),
              c: new BN(10000),
              bound: new BN(300000),
              which: IModifyCurveType.SwapSell
            }
          ],
          kept_fee: 100000,
          max_total_sold: new BN(10_000_000_000)
        },
        strategies: [
          {
            collateral_ratio: new BN(900000),
            liquidation_threshold: new BN(95000000),
            lend: true,
            swap: false,
            trade: false
          },
          {
            collateral_ratio: new BN(800000),
            liquidation_threshold: new BN(85000000),
            lend: true,
            swap: true,
            trade: false
          },
          {
            collateral_ratio: new BN(600000),
            liquidation_threshold: new BN(70000000),
            lend: true,
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
