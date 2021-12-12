use halo2::{
    circuit::Layouter,
    plonk::{Advice, Column, ConstraintSystem, Error},
};

use crate::gates::base_conversion::BaseConversionConfig;
use crate::gates::tables::BaseInfo;
use pairing::arithmetic::FieldExt;
use std::convert::TryInto;

struct StateBaseConversion<F> {
    bi: BaseInfo<F>,
    bccs: [BaseConversionConfig<F>; 25],
    state: [Column<Advice>; 25],
}

impl<F: FieldExt> StateBaseConversion<F> {
    fn configure(
        meta: &mut ConstraintSystem<F>,
        state: [Column<Advice>; 25],
        bi: BaseInfo<F>,
    ) -> Self {
        let bccs: [BaseConversionConfig<F>; 25] = state
            .iter()
            .map(|&lane| {
                BaseConversionConfig::configure(meta, bi.clone(), lane)
            })
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();

        Self { bi, bccs, state }
    }

    pub fn assign_region(
        &self,
        layouter: &mut impl Layouter<F>,
        input_state: [F; 25],
    ) -> Result<[F; 25], Error> {
        let output_state: Result<Vec<F>, Error> = input_state
            .iter()
            .zip(self.bccs.iter())
            .map(|(&lane, config)| {
                let output = config.assign_region(layouter, lane)?;
                Ok(output)
            })
            .into_iter()
            .collect();
        let output_state = output_state?;
        let output_state: [F; 25] = output_state.try_into().unwrap();
        Ok(output_state)
    }
}
