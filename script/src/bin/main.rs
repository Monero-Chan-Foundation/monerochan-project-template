use alloy_sol_types::SolType;
use clap::Parser;
use fibonacci_lib::PublicValuesStruct;
use monerochan::{include_elf, ProverClient, MONEROCHANStdin};

pub const FIBONACCI_ELF: &[u8] = include_elf!("fibonacci-program");

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long)]
    execute: bool,

    #[arg(long)]
    prove: bool,

    #[arg(long, default_value = "10")]
    n: u32,
}

fn main() {
    monerochan::utils::setup_logger();
    dotenv::dotenv().ok();

    let args = Args::parse();

    if args.execute == args.prove {
        eprintln!("Error: You must specify either --execute or --prove");
        std::process::exit(1);
    }

    // Check which prover mode is being used
    let prover_mode = std::env::var("MONEROCHAN_PROVER").unwrap_or_else(|_| "cpu".to_string());
    eprintln!("Using prover mode: {}", prover_mode);
    
    // Check for network authentication when using network prover
    if prover_mode == "network" {
        if std::env::var("MONEROCHAN_NETWORK_PRIVATE_KEY").is_err() {
            eprintln!("Warning: MONEROCHAN_NETWORK_PRIVATE_KEY not set. Network proving may fail for non-exempt clients.");
            eprintln!("Set MONEROCHAN_NETWORK_PRIVATE_KEY to your Solana private key (hex or base58) for authentication.");
        } else {
            eprintln!("Network authentication: MONEROCHAN_NETWORK_PRIVATE_KEY is set");
        }
    }

    let client = ProverClient::from_env();

    let mut stdin = MONEROCHANStdin::new();
    stdin.write(&args.n);

    println!("n: {}", args.n);

    if args.execute {
        let (output, report) = client.execute(FIBONACCI_ELF, &stdin).run().unwrap();
        println!("Program executed successfully on monerochan.rs Runtime.");

        let decoded = PublicValuesStruct::abi_decode(output.as_slice()).unwrap();
        let PublicValuesStruct { n, a, b } = decoded;
        println!("n: {}", n);
        println!("a: {}", a);
        println!("b: {}", b);

        let (expected_a, expected_b) = fibonacci_lib::fibonacci(n);
        assert_eq!(a, expected_a);
        assert_eq!(b, expected_b);
        println!("Values are correct!");

        println!("Number of monerochan.rs Runtime cycles: {}", report.total_instruction_count());
    } else {
        let (pk, vk) = client.setup(FIBONACCI_ELF);

        let proof = client
            .prove(&pk, &stdin)
            .run()
            .expect("failed to generate private proof with monerochan.rs Runtime");

        println!("Successfully generated private proof with monerochan.rs Runtime!");

        client.verify(&proof, &vk).expect("failed to verify private proof with monerochan.rs Runtime");
        println!("Successfully verified private proof with monerochan.rs Runtime!");
    }
}
