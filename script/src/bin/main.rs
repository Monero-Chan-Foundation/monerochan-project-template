use alloy_sol_types::SolType;
use clap::Parser;
use fibonacci_lib::PublicValuesStruct;
use hex;
use monerochan::{include_elf, network::NetworkMode, Prover, ProverClient, MONEROCHANStdin, NetworkProver};

pub const FIBONACCI_ELF: &[u8] = include_elf!("fibonacci-program");

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long)]
    execute: bool,

    #[arg(long)]
    prove: bool,

    #[arg(long)]
    async_prove: bool,

    #[arg(long, default_value = "10")]
    n: u32,

    #[arg(long, value_enum)]
    network_mode: Option<NetworkModeArg>,
}

#[derive(clap::ValueEnum, Clone, Debug)]
enum NetworkModeArg {
    Mainnet,
    Reserved,
}

fn main() {
    monerochan::utils::setup_logger();
    dotenv::dotenv().ok();

    let args = Args::parse();

    if args.execute == args.prove && !args.async_prove {
        eprintln!("Error: You must specify either --execute, --prove, or --async-prove");
        std::process::exit(1);
    }

    // Check which prover mode is being used
    let prover_mode = std::env::var("MONEROCHAN_PROVER").unwrap_or_else(|_| "cpu".to_string());
    eprintln!("Using prover mode: {}", prover_mode);
    
    // Check for network authentication when using network prover
    if prover_mode == "network" {
        let has_monerochan_key = std::env::var("MONEROCHAN_NETWORK_PRIVATE_KEY").is_ok();
        let has_base_key = std::env::var("BASE_PRIVATE_KEY").is_ok();
        
        if !has_monerochan_key && !has_base_key {
            eprintln!("Warning: Neither MONEROCHAN_NETWORK_PRIVATE_KEY nor BASE_PRIVATE_KEY is set.");
            eprintln!("Network proving may fail for non-exempt clients.");
            eprintln!("Set MONEROCHAN_NETWORK_PRIVATE_KEY (or BASE_PRIVATE_KEY) to your Solana private key (hex or base58) for authentication.");
        } else {
            if has_monerochan_key {
            eprintln!("Network authentication: MONEROCHAN_NETWORK_PRIVATE_KEY is set");
            } else {
                eprintln!("Network authentication: BASE_PRIVATE_KEY is set");
            }
        }
    }

    // Build client with explicit network mode if specified
    // For async_prove, we need a network prover, so ensure we use network mode
    enum ClientType {
        Network(NetworkProver),
        Env(monerochan::EnvProver),
    }
    
    let client = if args.async_prove {
        // Async proving requires network mode
        let mode = if let Some(mode_arg) = args.network_mode {
            match mode_arg {
                NetworkModeArg::Mainnet => NetworkMode::Mainnet,
                NetworkModeArg::Reserved => NetworkMode::Reserved,
            }
        } else {
            // Default to Reserved if not specified
            NetworkMode::Reserved
        };
        eprintln!("Using network prover with mode: {:?} (required for async proving)", mode);
        ClientType::Network(ProverClient::builder().network_for(mode).build())
    } else if let Some(mode_arg) = args.network_mode {
        let mode = match mode_arg {
            NetworkModeArg::Mainnet => NetworkMode::Mainnet,
            NetworkModeArg::Reserved => NetworkMode::Reserved,
        };
        eprintln!("Using explicit network mode: {:?}", mode);
        ClientType::Network(ProverClient::builder().network_for(mode).build())
    } else {
        ClientType::Env(ProverClient::from_env())
    };

    let mut stdin = MONEROCHANStdin::new();
    stdin.write(&args.n);

    println!("n: {}", args.n);

    match client {
        ClientType::Network(network_client) => {
            if args.execute {
                let (output, report) = network_client.execute(FIBONACCI_ELF, &stdin).run().unwrap();
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
            } else if args.async_prove {
                // Async proof example using request_async + wait_proof pattern
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    let (pk, vk) = network_client.setup(FIBONACCI_ELF);

                    println!("Submitting proof request to network...");
                    let request_id = network_client
                        .prove(&pk, &stdin)
                        .request_async()
                        .await
                        .expect("failed to submit proof request");

                    println!("Proof request submitted. Request ID: {}", hex::encode(request_id.as_slice()));
                    println!("Waiting for proof to complete...");

                    let proof = network_client
                        .wait_proof(request_id, None, None)
                        .await
                        .expect("failed to wait for proof");

                    println!("Successfully generated proof with monerochan.rs Runtime!");

                    network_client.verify(&proof, &vk).expect("failed to verify proof with monerochan.rs Runtime");
                    println!("Successfully verified proof with monerochan.rs Runtime!");
                });
            } else {
                let (pk, vk) = network_client.setup(FIBONACCI_ELF);

                let proof = network_client
                    .prove(&pk, &stdin)
                    .run()
                    .expect("failed to generate proof with monerochan.rs Runtime");

                println!("Successfully generated proof with monerochan.rs Runtime!");

                network_client.verify(&proof, &vk).expect("failed to verify proof with monerochan.rs Runtime");
                println!("Successfully verified proof with monerochan.rs Runtime!");
            }
        }
        ClientType::Env(env_client) => {
    if args.execute {
                let (output, report) = env_client.execute(FIBONACCI_ELF, &stdin).run().unwrap();
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
                let (pk, vk) = env_client.setup(FIBONACCI_ELF);

                let proof = env_client
            .prove(&pk, &stdin)
            .run()
            .expect("failed to generate private proof with monerochan.rs Runtime");

        println!("Successfully generated private proof with monerochan.rs Runtime!");

                env_client.verify(&proof, &vk).expect("failed to verify private proof with monerochan.rs Runtime");
        println!("Successfully verified private proof with monerochan.rs Runtime!");
            }
        }
    }
}
