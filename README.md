# monerochan.rs Project Template

This is a template for creating an end-to-end [monerochan.rs](https://github.com/Monero-Chan-Foundation/monerochan-rs) project that can generate a proof of any RISC-V program using the monerochan.rs Runtime.

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

## Network Authentication (Release 1)

When using the network prover (`MONEROCHAN_PROVER=network`), client authentication is required for non-exempt clients.

### Setup

1. Generate a Solana keypair:
   ```sh
   cd scripts
   ./generate_solana_keypair.sh
   ```
   
   This will:
   - Generate a new Solana keypair
   - Display the private key (hex) and Solana address
   - Automatically create/update `.env` with the actual `MONEROCHAN_NETWORK_PRIVATE_KEY`
   - Create/update `.env.example` with a placeholder (for version control)

2. Register your client address with the network:
   - Copy the Solana address from the script output
   - Contact network administrators to register your address
   - Fee-exempt clients (enterprise/sponsored) skip authentication

3. Load the environment variables:
   ```sh
   # The script automatically creates .env with your key
   # Source it to use:
   source .env
   
   # Or use with your application (dotenv will load it automatically)
   ```

### Usage

The SDK automatically handles authentication when `MONEROCHAN_NETWORK_PRIVATE_KEY` is set:

```sh
export MONEROCHAN_NETWORK_PRIVATE_KEY="0x..."
export MONEROCHAN_PROVER=network
cd script
cargo run --release -- --prove
```

Authentication includes:
- Ed25519 signature verification
- Timestamp validation (1 hour expiration)
- Whitelist enforcement

### Example

If you've generated a keypair using the script above, the `.env` file is automatically loaded:

```sh
export MONEROCHAN_PROVER=network
cd script
cargo run --release -- --prove --n 10
```

Or export the key directly:

```sh
export MONEROCHAN_NETWORK_PRIVATE_KEY="0x..."
export MONEROCHAN_PROVER=network
cd script
cargo run --release -- --prove --n 10
```