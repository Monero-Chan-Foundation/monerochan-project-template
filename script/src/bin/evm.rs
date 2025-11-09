use alloy_sol_types::SolType;
use clap::{Parser, ValueEnum};
use fibonacci_lib::PublicValuesStruct;
use serde::{Deserialize, Serialize};
use monerochan::{
    include_elf, HashableKey, ProverClient, MONEROCHANProofWithPublicValues, MONEROCHANStdin, MONEROCHANVerifyingKey,
};
use std::path::PathBuf;

pub const FIBONACCI_ELF: &[u8] = include_elf!("fibonacci-program");

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct EVMArgs {
    #[arg(long, default_value = "20")]
    n: u32,
    #[arg(long, value_enum, default_value = "groth16")]
    system: ProofSystem,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
enum ProofSystem {
    Plonk,
    Groth16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MONEROCHANFibonacciProofFixture {
    a: u32,
    b: u32,
    n: u32,
    vkey: String,
    public_values: String,
    proof: String,
}

fn main() {
    monerochan::utils::setup_logger();

    let args = EVMArgs::parse();

    let client = ProverClient::from_env();

    let (pk, vk) = client.setup(FIBONACCI_ELF);

    let mut stdin = MONEROCHANStdin::new();
    stdin.write(&args.n);

    println!("n: {}", args.n);
    println!("Proof System: {:?}", args.system);

    let proof = match args.system {
        ProofSystem::Plonk => client.prove(&pk, &stdin).plonk().run(),
        ProofSystem::Groth16 => client.prove(&pk, &stdin).groth16().run(),
    }
    .expect("failed to generate proof");

    create_proof_fixture(&proof, &vk, args.system);
}

fn create_proof_fixture(
    proof: &MONEROCHANProofWithPublicValues,
    vk: &MONEROCHANVerifyingKey,
    system: ProofSystem,
) {
    let bytes = proof.public_values.as_slice();
    let PublicValuesStruct { n, a, b } = PublicValuesStruct::abi_decode(bytes).unwrap();

    let fixture = MONEROCHANFibonacciProofFixture {
        a,
        b,
        n,
        vkey: vk.bytes32().to_string(),
        public_values: format!("0x{}", hex::encode(bytes)),
        proof: format!("0x{}", hex::encode(proof.bytes())),
    };

    println!("Verification Key: {}", fixture.vkey);

    println!("Public Values: {}", fixture.public_values);

    println!("Proof Bytes: {}", fixture.proof);

    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../contracts/src/fixtures");
    std::fs::create_dir_all(&fixture_path).expect("failed to create fixture path");
    std::fs::write(
        fixture_path.join(format!("{:?}-fixture.json", system).to_lowercase()),
        serde_json::to_string_pretty(&fixture).unwrap(),
    )
    .expect("failed to write fixture");
}
