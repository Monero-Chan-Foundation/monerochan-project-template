#![no_main]
monerochan_runtime::entrypoint!(main);

use alloy_sol_types::SolType;
use fibonacci_lib::{fibonacci, PublicValuesStruct};

pub fn main() {
    let n = monerochan_runtime::io::read::<u32>();

    let (a, b) = fibonacci(n);

    let bytes = PublicValuesStruct::abi_encode(&PublicValuesStruct { n, a, b });

    monerochan_runtime::io::commit_slice(&bytes);
}
