use monerochan::{include_elf, HashableKey, Prover, ProverClient};

pub const FIBONACCI_ELF: &[u8] = include_elf!("fibonacci-program");

fn main() {
    let prover = ProverClient::builder().cpu().build();
    let (_, vk) = prover.setup(FIBONACCI_ELF);
    println!("{}", vk.bytes32());
}
