use crate::gates::gate_helpers::CellF;
use pairing::arithmetic::FieldExt;

use halo2::{
    circuit::{Layouter, Region},
    plonk::{Advice, Column, ConstraintSystem, Error, Fixed, Selector},
    poly::Rotation,
};
use std::marker::PhantomData;

#[derive(Debug, Clone)]
pub struct BaseEvaluationConfig<F> {
    q_enable: Selector,
    base: u64,
    pub coef: Column<Advice>,
    power_of_base: Column<Fixed>,
    acc: Column<Advice>,
    _marker: PhantomData<F>,
}

impl<F: FieldExt> BaseEvaluationConfig<F> {
    /// # Side effect
    /// Enable equality on result and acc
    pub fn configure(
        meta: &mut ConstraintSystem<F>,
        base: u64,
        result: Column<Advice>,
    ) -> Self {
        let q_enable = meta.selector();
        let coef = meta.advice_column();
        let power_of_base = meta.fixed_column();
        let acc = meta.advice_column();
        meta.enable_equality(result.into());
        meta.enable_equality(acc.into());

        meta.create_gate("Running sum", |meta| {
            let q_enable = meta.query_selector(q_enable);
            let coef = meta.query_advice(coef, Rotation::cur());
            let power_of_base =
                meta.query_fixed(power_of_base, Rotation::cur());
            let acc_next = meta.query_advice(acc, Rotation::next());
            let acc = meta.query_advice(acc, Rotation::cur());

            // acc_{i+1} = acc_{i} * base ** power + coef
            vec![(
                "running sum",
                q_enable * (acc_next - acc * power_of_base - coef),
            )]
        });

        Self {
            q_enable,
            base,
            coef,
            power_of_base,
            acc,
            _marker: PhantomData,
        }
    }

    pub fn assign_region(
        &self,
        layouter: &mut impl Layouter<F>,
        result: CellF<F>,
        coefs: &Vec<F>,
        power_of_bases: &Vec<F>,
    ) -> Result<(), Error> {
        layouter.assign_region(
            || "Base eval",
            |mut region| {
                let mut acc = F::zero();
                for (offset, (&pob, &coef)) in
                    power_of_bases.iter().zip(coefs).enumerate()
                {
                    region.assign_advice(
                        || "Coef",
                        self.coef,
                        offset,
                        || Ok(coef),
                    )?;
                    region.assign_fixed(
                        || "Power of base",
                        self.power_of_base,
                        offset,
                        || Ok(pob),
                    )?;
                    let acc_cell = region.assign_advice(
                        || "Acc",
                        self.acc,
                        offset,
                        || Ok(acc),
                    )?;
                    if offset == 0 {
                        region.constrain_constant(acc_cell, F::zero())?;
                    }
                    acc = acc * pob + coef;
                }
                let final_acc = region.assign_advice(
                    || "Final Acc",
                    self.acc,
                    coefs.len(),
                    || Ok(acc),
                )?;

                region.constrain_equal(final_acc, result.cell)?;

                Ok(())
            },
        )?;
        Ok(())
    }
}
