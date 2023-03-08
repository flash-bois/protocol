use crate::{
    core_lib::{
        decimal::{Factories, Fraction, Price, Quantity, Utilization},
        errors::LibErrors,
        structs::FeeCurve,
    },
    structs::{State, Vaults},
};
use anchor_lang::prelude::*;
use checked_decimal_macro::{BetweenDecimals, Decimal};

#[derive(Accounts)]
pub struct Admin<'info> {
    #[account(mut, seeds = [b"state".as_ref()], bump=state.load()?.bump)]
    pub state: AccountLoader<'info, State>,
    #[account(mut, constraint = vaults.key() == state.load()?.vaults_acc)]
    pub vaults: AccountLoader<'info, Vaults>,
    #[account(mut, constraint = admin.key() == state.load()?.admin)]
    pub admin: Signer<'info>,
}

impl Admin<'_> {
    pub fn force_override_oracle(
        &mut self,
        index: u8,
        base: bool,
        price: u32,
        conf: u32,
        exp: i8,
        time: Option<u32>, // can be overridden as well
    ) -> anchor_lang::Result<()> {
        msg!("DotWave: Force override oracle");

        let vaults = &mut self.vaults.load_mut()?;
        let vault = vaults.vault_checked_mut(index)?;

        let oracle = match base {
            true => vault.oracle_mut(),
            false => vault.quote_oracle_mut(),
        }?;

        let time = time.unwrap_or(
            Clock::get()?
                .unix_timestamp
                .try_into()
                .map_err(|_| LibErrors::ParseError)?,
        );

        let (price, confidence) = if exp < 0 {
            (
                Price::from_scale(
                    price,
                    exp.abs().try_into().map_err(|_| LibErrors::ParseError)?,
                ),
                Price::from_scale(
                    conf,
                    exp.abs().try_into().map_err(|_| LibErrors::ParseError)?,
                ),
            )
        } else {
            (
                Price::from_integer(price)
                    / Price::from_scale(1, exp.try_into().map_err(|_| LibErrors::ParseError)?),
                Price::from_integer(conf)
                    / Price::from_scale(1, exp.try_into().map_err(|_| LibErrors::ParseError)?),
            )
        };

        oracle.update(price, confidence, time)?;

        Ok(())
    }

    pub fn enable_lending(
        &mut self,
        index: u8,
        max_utilization: u32,
        max_total_borrow: u64,
        initial_fee_time: u32,
    ) -> anchor_lang::Result<()> {
        msg!("DotWave: Enabling lending");

        let vaults = &mut self.vaults.load_mut()?;
        let vault = vaults.vault_checked_mut(index)?;

        vault.enable_lending(
            FeeCurve::default(),
            Utilization::from_decimal(Fraction::new(max_utilization as u64)),
            Quantity::new(max_total_borrow),
            initial_fee_time,
            Clock::get()?.unix_timestamp as u32,
        )?;

        Ok(())
    }

    pub fn enable_swapping(
        &mut self,
        index: u8,
        kept_fee: u32,
        _max_total_sold: u64,
    ) -> anchor_lang::Result<()> {
        msg!("DotWave: Enabling swapping");

        let vaults = &mut self.vaults.load_mut()?;
        let vault = vaults.vault_checked_mut(index)?;

        vault.enable_swapping(
            FeeCurve::default(),
            FeeCurve::default(),
            Fraction::new(kept_fee as u64),
        )?;

        Ok(())
    }

    pub fn modify_fee_curve(
        &self,
        vault: u8,
        service: u8,
        base: bool,
        bound: u64,
        a: u64,
        b: u64,
        c: u64,
    ) -> Result<()> {
        msg!("DotWave: Modify fee curve");

        let vaults = &mut self.vaults.load_mut()?;
        let vault = vaults.vault_checked_mut(vault)?;

        let curve = match (service, base) {
            (1, true) => vault.lend_service()?.fee_curve(),
            (2, true) => vault.swap_service()?.fee_curve_sell(),
            (2, false) => vault.swap_service()?.fee_curve_buy(),
            _ => return Err(LibErrors::InvalidService.into()),
        };

        let bound = Fraction::new(bound);

        match (a, b, c) {
            (0, 0, c) => curve.add_constant_fee(Fraction::new(c), bound),
            (0, b, c) => curve.add_linear_fee(Fraction::new(c), Fraction::new(b), bound),
            _ => unimplemented!("fee not yet implemented"),
        };

        Ok(())
    }
}
