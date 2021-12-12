use halo2::{
    circuit::Layouter,
    plonk::{Advice, Column, ConstraintSystem, Error, Selector},
    poly::Rotation,
};

use crate::gates::base_eval::BaseEvaluationConfig;
use crate::gates::gate_helpers::CellF;
use crate::gates::tables::BaseInfo;
use pairing::arithmetic::FieldExt;

#[derive(Clone, Debug)]
pub struct BaseConversionConfig<F> {
    q_enable: Selector,
    bi: BaseInfo<F>,
    input_eval: BaseEvaluationConfig<F>,
    output_eval: BaseEvaluationConfig<F>,
}

impl<F: FieldExt> BaseConversionConfig<F> {
    /// Side effect: input_lane and output_lane are equality enabled
    pub fn configure(
        meta: &mut ConstraintSystem<F>,
        bi: BaseInfo<F>,
        input_lane: Column<Advice>,
        output_lane: Column<Advice>,
    ) -> Self {
        let q_enable = meta.complex_selector();

        let input_eval =
            BaseEvaluationConfig::configure(meta, bi.input_pob(), input_lane);
        let output_eval =
            BaseEvaluationConfig::configure(meta, bi.output_pob(), output_lane);

        meta.lookup(|meta| {
            let q_enable = meta.query_selector(q_enable);
            let input_slices =
                meta.query_advice(input_eval.coef, Rotation::cur());
            let output_slices =
                meta.query_advice(output_eval.coef, Rotation::cur());
            vec![
                (q_enable.clone() * input_slices, bi.input_tc),
                (q_enable * output_slices, bi.output_tc),
            ]
        });

        Self {
            q_enable,
            bi,
            input_eval,
            output_eval,
        }
    }

    pub fn assign_region(
        &self,
        layouter: &mut impl Layouter<F>,
        input_lane: CellF<F>,
        output_lane: CellF<F>,
    ) -> Result<(), Error> {
        let (input_coefs, output_coefs) =
            self.bi.compute_coefs(input_lane.value)?;
        self.input_eval
            .assign_region(layouter, input_lane, &input_coefs)?;
        self.output_eval
            .assign_region(layouter, output_lane, &output_coefs)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arith_helpers::convert_b2_to_b13;
    use crate::gates::{
        gate_helpers::biguint_to_f, tables::FromBinaryTableConfig,
    };
    use halo2::{
        circuit::{Layouter, SimpleFloorPlanner},
        dev::MockProver,
        plonk::{Advice, Circuit, Column, ConstraintSystem, Error},
    };
    use pairing::arithmetic::FieldExt;
    use pairing::bn256::Fr as Fp;
    use pretty_assertions::assert_eq;
    #[test]
    fn test_base_conversion() {
        #[derive(Debug, Clone)]
        struct MyConfig<F> {
            input_lane: Column<Advice>,
            output_lane: Column<Advice>,
            table: FromBinaryTableConfig<F>,
            conversion: BaseConversionConfig<F>,
        }
        impl<F: FieldExt> MyConfig<F> {
            pub fn configure(meta: &mut ConstraintSystem<F>) -> Self {
                let table = FromBinaryTableConfig::configure(meta);
                let input_lane = meta.advice_column();
                let output_lane = meta.advice_column();
                let bi = table.get_base_info(false);
                let conversion = BaseConversionConfig::configure(
                    meta,
                    bi,
                    input_lane,
                    output_lane,
                );
                Self {
                    input_lane,
                    output_lane,
                    table,
                    conversion,
                }
            }

            pub fn load(
                &self,
                layouter: &mut impl Layouter<F>,
            ) -> Result<(), Error> {
                self.table.load(layouter)
            }

            pub fn assign_region(
                &self,
                layouter: &mut impl Layouter<F>,
                input_value: F,
                output_value: F,
            ) -> Result<(), Error> {
                let (input_lane, output_lane) = layouter.assign_region(
                    || "I/O values",
                    |mut region| {
                        let input_lane = region.assign_advice(
                            || "input lane",
                            self.input_lane,
                            0,
                            || Ok(input_value),
                        )?;
                        let output_lane = region.assign_advice(
                            || "output lane",
                            self.output_lane,
                            0,
                            || Ok(output_value),
                        )?;

                        Ok((
                            CellF {
                                cell: input_lane,
                                value: input_value,
                            },
                            CellF {
                                cell: output_lane,
                                value: output_value,
                            },
                        ))
                    },
                )?;
                self.conversion.assign_region(
                    layouter,
                    input_lane,
                    output_lane,
                )?;
                Ok(())
            }
        }

        #[derive(Default)]
        struct MyCircuit<F> {
            input_b2_lane: F,
            output_b13_lane: F,
        }
        impl<F: FieldExt> Circuit<F> for MyCircuit<F> {
            type Config = MyConfig<F>;
            type FloorPlanner = SimpleFloorPlanner;

            fn without_witnesses(&self) -> Self {
                Self::default()
            }

            fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
                Self::Config::configure(meta)
            }

            fn synthesize(
                &self,
                config: Self::Config,
                mut layouter: impl Layouter<F>,
            ) -> Result<(), Error> {
                config.load(&mut layouter)?;
                config.assign_region(
                    &mut layouter,
                    self.input_b2_lane,
                    self.output_b13_lane,
                )?;
                Ok(())
            }
        }
        let input = 12345678u64;
        let circuit = MyCircuit::<Fp> {
            input_b2_lane: Fp::from(input),
            output_b13_lane: biguint_to_f::<Fp>(&convert_b2_to_b13(input))
                .unwrap(),
        };
        let prover = MockProver::<Fp>::run(17, &circuit, vec![]).unwrap();
        assert_eq!(prover.verify(), Ok(()));
    }
}
