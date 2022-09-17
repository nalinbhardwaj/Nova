use crate::{circom_circuit::CircomCircuit, circom_reader::{load_r1cs, load_witness_from_file}};
use blstrs::Bls12;
use nova_snark::{traits::circuit::TrivialTestCircuit, PublicParams};
use pairing::Engine;

mod circom_circuit;
mod circom_file;
mod circom_reader;

fn main() {
  let circuit_file = "/Users/nibnalin/Documents/Nova/examples/circom/artifacts/main.r1cs";
  let witness_file = "/Users/nibnalin/Documents/Nova/examples/circom/artifacts/witness.wtns";

  let circuit_primary = CircomCircuit {
      r1cs: load_r1cs(&circuit_file),
      witness: Some(load_witness_from_file::<<Bls12 as Engine>::Fr>(&witness_file)),
      wire_mapping: None,
      aux_offset: 1,
  };

  let circuit_secondary = TrivialTestCircuit::default();
  let pp = PublicParams::<
    G1,
    G2,
    MinRootCircuit<<G1 as Group>::Scalar>,
    TrivialTestCircuit<<G2 as Group>::Scalar>,
  >::setup(circuit_primary, circuit_secondary.clone());
  println!(
    "Number of constraints per step (primary circuit): {}",
    pp.num_constraints().0
  );
  println!(
    "Number of constraints per step (secondary circuit): {}",
    pp.num_constraints().1
  );

  println!("test");
}