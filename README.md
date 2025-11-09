# Monero-Chan Project Template

This is a template for creating an end-to-end [monerochan.rs](https://github.com/Monero-Chan-Foundation/monerochan-rs) project that can generate a proof of any RISC-V program using the Monero-Chan RISC-V Runtime.

## Requirements

- [Rust](https://rustup.rs/)
- [monerochan.rs](https://github.com/Monero-Chan-Foundation/monerochan-rs)

## Running the Project

There are 2 main ways to run this project: execute a program and generate a private proof.

### Build the Program

The program is automatically built through `script/build.rs` when the script is built.

### Execute the Program

To run the program without generating a proof:

```sh
cd script
cargo run --release -- --execute
```

This will execute the program and display the output.

### Generate a Private Proof

To generate a private proof for your program:

```sh
cd script
cargo run --release -- --prove
```

### Retrieve the Verification Key

To retrieve your `programVKey`, run the following command in `script`:

```sh
cargo run --release --bin vkey
```
