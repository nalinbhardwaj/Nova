extern crate rand;

use bellperson::gadgets::num::AllocatedNum;
use itertools::Itertools;
use nova_snark::traits::circuit::StepCircuit;
use serde::{Serialize, Deserialize};
use std::collections::BTreeMap;
use std::str;

use ::bellperson::{
    Circuit, ConstraintSystem, Index, LinearCombination, SynthesisError, Variable,
};
use ff::PrimeField;
pub use num_bigint::BigUint;


#[derive(Serialize, Deserialize)]
pub struct CircuitJson {
    pub constraints: Vec<Vec<BTreeMap<String, String>>>,
    #[serde(rename = "nPubInputs")]
    pub num_inputs: usize,
    #[serde(rename = "nOutputs")]
    pub num_outputs: usize,
    #[serde(rename = "nVars")]
    pub num_variables: usize,
}

pub type Constraint<Fr> = (
    Vec<(usize, Fr)>,
    Vec<(usize, Fr)>,
    Vec<(usize, Fr)>,
);

#[derive(Clone)]
pub struct R1CS<Fr: PrimeField> {
    pub num_inputs: usize,
    pub num_aux: usize,
    pub num_variables: usize,
    pub constraints: Vec<Constraint<Fr>>,
}

#[derive(Clone)]
pub struct CircomCircuit<Fr: PrimeField> {
    pub r1cs: R1CS<Fr>,
    pub witness: Option<Vec<Fr>>,
    pub wire_mapping: Option<Vec<usize>>,
    pub aux_offset: usize,
    // debug symbols
}

impl<'a, Fr: PrimeField> CircomCircuit<Fr> {
    pub fn get_public_inputs(&self) -> Option<Vec<Fr>> {
        match &self.witness {
            None => None,
            Some(w) => match &self.wire_mapping {
                None => Some(w[1..self.r1cs.num_inputs].to_vec()),
                Some(m) => Some(m[1..self.r1cs.num_inputs].iter().map(|i| w[*i]).collect_vec()),
            },
        }
    }

    pub fn vanilla_synthesize<CS: ConstraintSystem<Fr>>(&self, cs: &mut CS) -> Result<(), SynthesisError> {
        let witness = &self.witness;
        let wire_mapping = &self.wire_mapping;
        for i in 1..self.r1cs.num_inputs {
            cs.alloc_input(
                || format!("variable {}", i),
                || {
                    Ok(match witness {
                        None => Fr::one(),
                        Some(w) => match wire_mapping {
                            None => w[i],
                            Some(m) => w[m[i]],
                        },
                    })
                },
            )?;
        }
        for i in 0..self.r1cs.num_aux {
            cs.alloc(
                || format!("aux {}", i + self.aux_offset),
                || {
                    Ok(match witness {
                        None => Fr::one(),
                        Some(w) => match wire_mapping {
                            None => w[i + self.r1cs.num_inputs],
                            Some(m) => w[m[i + self.r1cs.num_inputs]],
                        },
                    })
                },
            )?;
        }

        let make_index = |index| {
            if index < self.r1cs.num_inputs {
                Index::Input(index)
            } else {
                Index::Aux(index - self.r1cs.num_inputs + self.aux_offset)
            }
        };
        let make_lc = |lc_data: Vec<(usize, Fr)>| {
            lc_data
                .iter()
                .fold(LinearCombination::<Fr>::zero(), |lc: LinearCombination<Fr>, (index, coeff)| {
                    lc + (*coeff, Variable::new_unchecked(make_index(*index)))
                })
        };
        for (i, constraint) in self.r1cs.constraints.iter().enumerate() {
            // 0 * LC = 0 must be ignored
            if !((constraint.0.is_empty() || constraint.1.is_empty()) && constraint.2.is_empty()) {
                cs.enforce(
                    || format!("{}", i),
                    |_| make_lc(constraint.0.clone()),
                    |_| make_lc(constraint.1.clone()),
                    |_| make_lc(constraint.2.clone()),
                );
            }
        }
        Ok(())
    }
}

impl<'a, Fr: PrimeField> Circuit<Fr> for CircomCircuit<Fr> {
    fn synthesize<CS: ConstraintSystem<Fr>>(self, cs: &mut CS) -> Result<(), SynthesisError> {
        self.vanilla_synthesize(cs)
    }
}


impl<'a, Fr: PrimeField> StepCircuit<Fr> for CircomCircuit<Fr> {
  fn arity(&self) -> usize {
    1
  }

  fn synthesize<CS: ConstraintSystem<Fr>>(
    &self,
    cs: &mut CS,
    z: &[AllocatedNum<Fr>],
  ) -> Result<Vec<AllocatedNum<Fr>>, SynthesisError> {
    let mut z_out: Result<Vec<AllocatedNum<Fr>>, SynthesisError> =
      Err(SynthesisError::AssignmentMissing);

    // use the provided inputs
    let x_0 = z[0].clone();
    z_out = Ok(vec![x_0.clone()]);

    // synthesize the circuit
    self.vanilla_synthesize(cs)?;
    
    z_out
  }

  fn output(&self, z: &[Fr]) -> Vec<Fr> {
    // // sanity check
    // debug_assert_eq!(z[0], self.seq[0].x_i);
    // debug_assert_eq!(z[1], self.seq[0].y_i);

    // compute output using advice
    vec![
      z[0]
    ]
  }
}
