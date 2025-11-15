#!/bin/bash
# Generate Solana keypair for network authentication
# This script generates a private key and derives the Solana address

set -e

# Get project root directory (parent of scripts directory)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "Generating Solana keypair for network authentication..."
echo ""

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "Error: cargo not found. Please install Rust: https://rustup.rs/"
    exit 1
fi

# Create a temporary Rust project to generate the keypair
TMP_DIR=$(mktemp -d)
trap "rm -rf $TMP_DIR" EXIT

cd "$TMP_DIR"
cargo init --name generate_keypair --quiet > /dev/null 2>&1

cat > Cargo.toml << 'EOF'
[package]
name = "generate_keypair"
version = "0.1.0"
edition = "2021"

[dependencies]
ed25519-dalek = "2.0"
bs58 = "0.5"
rand = { version = "0.8", features = ["std_rng"] }
hex = "0.4"
EOF

cat > src/main.rs << 'EOF'
use ed25519_dalek::SigningKey;
use bs58;
use rand::RngCore;
use rand::rngs::OsRng;

fn main() {
    // Generate random 32-byte private key
    let mut bytes = [0u8; 32];
    OsRng.fill_bytes(&mut bytes);
    
    let signing_key = SigningKey::from_bytes(&bytes);
    let verifying_key = signing_key.verifying_key();
    
    // Derive Solana address (base58 of public key)
    let address = bs58::encode(verifying_key.as_bytes()).into_string();
    
    // Output private key as hex
    let priv_key_hex = hex::encode(&bytes);
    
    println!("{}", priv_key_hex);
    println!("{}", address);
}
EOF

OUTPUT=$(cargo run --release 2>&1 | grep -v "^   Compiling\|^    Finished\|^     Running\|^    Updating\|^     Locking\|^      Adding\|^warning:" | tail -2)
PRIV_KEY=$(echo "$OUTPUT" | head -1)
ADDRESS=$(echo "$OUTPUT" | tail -1)

if [ -z "$PRIV_KEY" ] || [ -z "$ADDRESS" ]; then
    echo "Error: Failed to generate keypair"
    exit 1
fi

echo ""
echo "✅ Solana keypair generated!"
echo ""
echo "Private Key (hex): 0x$PRIV_KEY"
echo "Solana Address:     $ADDRESS"
echo ""

# Create or update .env file with actual key
cd "$PROJECT_ROOT"

ENV_FILE=".env"
ENV_EXAMPLE=".env.example"

# Update or add MONEROCHAN_NETWORK_PRIVATE_KEY to .env
if grep -q "^export MONEROCHAN_NETWORK_PRIVATE_KEY=" "$ENV_FILE" 2>/dev/null; then
    # Update existing entry
    if [[ "$OSTYPE" == "darwin"* ]]; then
        sed -i '' "s|^export MONEROCHAN_NETWORK_PRIVATE_KEY=.*|export MONEROCHAN_NETWORK_PRIVATE_KEY=\"0x$PRIV_KEY\"|" "$ENV_FILE"
    else
        sed -i "s|^export MONEROCHAN_NETWORK_PRIVATE_KEY=.*|export MONEROCHAN_NETWORK_PRIVATE_KEY=\"0x$PRIV_KEY\"|" "$ENV_FILE"
    fi
    echo "✅ Updated $ENV_FILE with new private key"
else
    # Append new entry
    echo "" >> "$ENV_FILE"
    echo "# Network Authentication (Release 1)" >> "$ENV_FILE"
    echo "export MONEROCHAN_NETWORK_PRIVATE_KEY=\"0x$PRIV_KEY\"" >> "$ENV_FILE"
    echo "✅ Added private key to $ENV_FILE"
fi

# Ensure .env.example has placeholders (not actual keys)
if [ ! -f "$ENV_EXAMPLE" ]; then
    # Create .env.example with all necessary variables
    cat > "$ENV_EXAMPLE" << 'EOF'
# Prover mode: cpu, gpu, or network
export MONEROCHAN_PROVER=cpu

# Network Authentication (Release 1)
# Generate your keypair with: ./scripts/generate_solana_keypair.sh
export MONEROCHAN_NETWORK_PRIVATE_KEY="0x..."
EOF
elif ! grep -q "^export MONEROCHAN_NETWORK_PRIVATE_KEY=" "$ENV_EXAMPLE" 2>/dev/null; then
    # Add Network Authentication section if it doesn't exist
    echo "" >> "$ENV_EXAMPLE"
    echo "# Network Authentication (Release 1)" >> "$ENV_EXAMPLE"
    echo "# Generate your keypair with: ./scripts/generate_solana_keypair.sh" >> "$ENV_EXAMPLE"
    echo "export MONEROCHAN_NETWORK_PRIVATE_KEY=\"0x...\"" >> "$ENV_EXAMPLE"
fi

# Ensure MONEROCHAN_PROVER is in .env.example
if ! grep -q "^export MONEROCHAN_PROVER=" "$ENV_EXAMPLE" 2>/dev/null; then
    # Add at the beginning if not present
    if [[ "$OSTYPE" == "darwin"* ]]; then
        sed -i '' '1i\
# Prover mode: cpu, gpu, or network\
export MONEROCHAN_PROVER=cpu\
' "$ENV_EXAMPLE"
    else
        sed -i '1i# Prover mode: cpu, gpu, or network\nexport MONEROCHAN_PROVER=cpu\n' "$ENV_EXAMPLE"
    fi
fi

echo ""
echo "⚠️  IMPORTANT: Keep your private key secure!"
echo "⚠️  Register your Solana address ($ADDRESS) with the network administrator."
echo ""

