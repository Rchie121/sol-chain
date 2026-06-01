#!/bin/bash
set -e

# Automate Anchor.toml program ID updates after `anchor build`
# Usage: ./scripts/update_anchor_ids.sh

ANCHOR_TOML="Anchor.toml"
DEPLOY_DIR="target/deploy"

if [ ! -f "$ANCHOR_TOML" ]; then
    echo "Error: $ANCHOR_TOML not found"
    exit 1
fi

if [ ! -d "$DEPLOY_DIR" ]; then
    echo "Error: $DEPLOY_DIR not found. Run 'anchor build' first."
    exit 1
fi

echo "Updating program IDs in $ANCHOR_TOML..."

# Function to get program ID from keypair
get_program_id() {
    local keypair="$1"
    if [ -f "$keypair" ]; then
        solana-keygen pubkey "$keypair" 2>/dev/null || echo ""
    else
        echo ""
    fi
}

# Update AMM
AMM_ID=$(get_program_id "$DEPLOY_DIR/amm-keypair.json")
if [ -n "$AMM_ID" ]; then
    # Update all occurrences of the placeholder or old ID
    sed -i.bak "s/AMM[0-9a-zA-Z]\{32,\}/$AMM_ID/g" "$ANCHOR_TOML"
    echo "  AMM Program ID → $AMM_ID"
else
    echo "  Warning: amm-keypair.json not found"
fi

# Update Lending
LEND_ID=$(get_program_id "$DEPLOY_DIR/lending-keypair.json")
if [ -n "$LEND_ID" ]; then
    sed -i.bak "s/LEND[0-9a-zA-Z]\{32,\}/$LEND_ID/g" "$ANCHOR_TOML"
    echo "  Lending Program ID → $LEND_ID"
else
    echo "  Warning: lending-keypair.json not found"
fi

# Clean up backup
rm -f "${ANCHOR_TOML}.bak"

echo "Done! Program IDs updated in $ANCHOR_TOML"
