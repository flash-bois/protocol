use anchor_lang::prelude::*;
use bytemuck::{Pod, Zeroable};
use std::cell::RefMut;
use std::ops::DerefMut;

pub const MAGIC: u32 = 0xa1b2c3d4;
pub const VERSION_2: u32 = 2;
pub const VERSION: u32 = VERSION_2;
pub const MAP_TABLE_SIZE: usize = 640;
pub const PROD_ACCT_SIZE: usize = 512;
pub const PROD_HDR_SIZE: usize = 48;
pub const PROD_ATTR_SIZE: usize = PROD_ACCT_SIZE - PROD_HDR_SIZE;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(C)]
pub enum CorpAction {
    NoCorpAct,
}

impl Default for CorpAction {
    fn default() -> Self {
        CorpAction::NoCorpAct
    }
}

/// The type of prices associated with a product -- each product may have multiple price feeds of
/// different types.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(C)]
pub enum PriceType {
    Unknown,
    Price,
}

impl Default for PriceType {
    fn default() -> Self {
        PriceType::Price
    }
}

/// Represents availability status of a price feed.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(C)]
pub enum PriceStatus {
    /// The price feed is not currently updating for an unknown reason.
    Unknown,
    /// The price feed is updating as expected.
    Trading,
    /// The price feed is not currently updating because trading in the product has been halted.
    Halted,
    /// The price feed is not currently updating because an auction is setting the price.
    Auction,
    /// A price component can be ignored if the confidence interval is too wide
    Ignored,
}

impl Default for PriceStatus {
    fn default() -> Self {
        PriceStatus::Trading
    }
}

/// A price and confidence at a specific slot. This struct can represent either a
/// publisher's contribution or the outcome of price aggregation.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
#[repr(C)]
pub struct PriceInfo {
    /// the current price.
    /// For the aggregate price use `get_price_no_older_than()` whenever possible. Accessing fields
    /// directly might expose you to stale or invalid prices.
    pub price: i64,
    /// confidence interval around the price.
    /// For the aggregate confidence use `get_price_no_older_than()` whenever possible. Accessing
    /// fields directly might expose you to stale or invalid prices.
    pub conf: u64,
    /// status of price (Trading is valid)
    pub status: PriceStatus,
    /// notification of any corporate action
    pub corp_act: CorpAction,
    pub pub_slot: u64,
}

/// The price and confidence contributed by a specific publisher.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
#[repr(C)]
pub struct PriceComp {
    /// key of contributing publisher
    pub publisher: Pubkey,
    /// the price used to compute the current aggregate price
    pub agg: PriceInfo,
    /// The publisher's latest price. This price will be incorporated into the aggregate price
    /// when price aggregation runs next.
    pub latest: PriceInfo,
}

/// An number represented as both `value` and also in rational as `numer/denom`.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
#[repr(C)]
pub struct Rational {
    pub val: i64,
    pub numer: i64,
    pub denom: i64,
}

/// Price accounts represent a continuously-updating price feed for a product.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
#[repr(C)]
pub struct PriceAccount {
    /// pyth magic number
    pub magic: u32,
    /// program version
    pub ver: u32,
    /// account type
    pub atype: u32,
    /// price account size
    pub size: u32,
    /// price or calculation type
    pub ptype: PriceType,
    /// price exponent
    pub expo: i32,
    /// number of component prices
    pub num: u32,
    /// number of quoters that make up aggregate
    pub num_qt: u32,
    /// slot of last valid (not unknown) aggregate price
    pub last_slot: u64,
    /// valid slot-time of agg. price
    pub valid_slot: u64,
    /// exponentially moving average price
    pub ema_price: Rational,
    /// exponentially moving average confidence interval
    pub ema_conf: Rational,
    /// unix timestamp of aggregate price
    pub timestamp: i64,
    /// min publishers for valid price
    pub min_pub: u8,
    /// space for future derived values
    pub drv2: u8,
    /// space for future derived values
    pub drv3: u16,
    /// space for future derived values
    pub drv4: u32,
    /// product account key
    pub prod: Pubkey,
    /// next Price account in linked list
    pub next: Pubkey,
    /// valid slot of previous update
    pub prev_slot: u64,
    /// aggregate price of previous update with TRADING status
    pub prev_price: i64,
    /// confidence interval of previous update with TRADING status
    pub prev_conf: u64,
    /// unix timestamp of previous aggregate with TRADING status
    pub prev_timestamp: i64,
    /// aggregate price info
    pub agg: PriceInfo,
    /// price components one per quoter
    pub comp: [PriceComp; 32],
}

#[cfg(target_endian = "little")]
unsafe impl Zeroable for PriceAccount {}

#[cfg(target_endian = "little")]
unsafe impl Pod for PriceAccount {}

impl PriceAccount {
    #[inline]
    pub fn load_mut<'a>(price_acc: &'a AccountInfo) -> Result<RefMut<'a, PriceAccount>> {
        let data = price_acc.try_borrow_mut_data()?;

        Ok(RefMut::map(data, |data| {
            bytemuck::from_bytes_mut(&mut data.deref_mut()[..std::mem::size_of::<Self>()])
        }))
    }
}

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod oracle {
    use pyth_sdk_solana::state::AccountType;

    use super::*;

    pub fn set(ctx: Context<Initialize>, price: i64, exp: i32, confidence: u64) -> Result<()> {
        let price_acc = &mut PriceAccount::load_mut(&ctx.accounts.price)?;

        **price_acc = PriceAccount::default();

        price_acc.magic = MAGIC;
        price_acc.ver = VERSION_2;
        price_acc.atype = AccountType::Price as u32;
        price_acc.timestamp = Clock::get()?.unix_timestamp;
        price_acc.agg.price = price;
        price_acc.agg.conf = confidence;
        price_acc.expo = exp;

        Ok(())
    }
}
#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    /// CHECK:`
    pub price: AccountInfo<'info>,
    #[account(mut)]
    pub signer: Signer<'info>,
}
