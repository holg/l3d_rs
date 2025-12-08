#!/usr/bin/env bash
# File: /Users/htr/Documents/develeop/rust/l3d-rs/scripts/test-ci-locally.sh

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${YELLOW}=== Testing CI Workflow Locally ===${NC}\n"

# Run cargo fmt check
echo -e "${YELLOW}Step 1: Running cargo fmt check...${NC}"
if cargo fmt --all -- --check; then
    echo -e "${GREEN}✓ cargo fmt passed${NC}\n"
else
    echo -e "${RED}✗ cargo fmt failed${NC}"
    echo -e "${YELLOW}Run 'cargo fmt --all' to fix formatting issues${NC}\n"
    exit 1
fi

# Run clippy
echo -e "${YELLOW}Step 2: Running clippy...${NC}"
if cargo clippy --workspace --all-targets -- -D warnings; then
    echo -e "${GREEN}✓ clippy passed${NC}\n"
else
    echo -e "${RED}✗ clippy failed${NC}\n"
    exit 1
fi

# Run cargo build
echo -e "${YELLOW}Step 3: Running cargo build...${NC}"
# Exclude l3d-python as it requires maturin to build
if cargo build --workspace --exclude l3d-python; then
    echo -e "${GREEN}✓ build passed${NC}\n"
else
    echo -e "${RED}✗ build failed${NC}\n"
    exit 1
fi

# Build l3d-python separately with maturin
echo -e "${YELLOW}Step 3b: Building l3d-python with maturin...${NC}"
if command -v maturin &> /dev/null; then
    if (cd crates/l3d-python && maturin build); then
        echo -e "${GREEN}✓ l3d-python build passed${NC}\n"
    else
        echo -e "${RED}✗ l3d-python build failed${NC}\n"
        exit 1
    fi
else
    echo -e "${YELLOW}⚠ maturin not installed, skipping l3d-python build${NC}"
    echo -e "${YELLOW}Install with: pip install maturin${NC}\n"
fi

# Run cargo doc
echo -e "${YELLOW}Step 4: Running cargo doc...${NC}"
if RUSTDOCFLAGS="-D warnings" cargo doc --workspace --document-private-items --no-deps; then
    echo -e "${GREEN}✓ doc generation passed${NC}\n"
else
    echo -e "${RED}✗ doc generation failed${NC}\n"
    exit 1
fi

# Additional checks
echo -e "${YELLOW}Step 5: Running additional checks...${NC}"

# Check cargo-sort FIRST (it should run before taplo)
if command -v cargo-sort &> /dev/null; then
    echo "Running cargo-sort check..."
    if cargo-sort -cwg; then
        echo -e "${GREEN}✓ cargo-sort passed${NC}"
    else
        echo -e "${RED}✗ cargo-sort failed${NC}"
        echo -e "${YELLOW}Run 'cargo-sort -wg' to fix Cargo.toml sorting${NC}"
        exit 1
    fi
else
    echo -e "${YELLOW}⚠ cargo-sort not installed, skipping Cargo.toml sort check${NC}"
    echo -e "${YELLOW}Install with: cargo install cargo-sort${NC}"
fi

# Check taplo AFTER cargo-sort (taplo formats what cargo-sort organized)
if command -v taplo &> /dev/null; then
    echo "Running taplo format check..."
    if taplo format --check; then
        echo -e "${GREEN}✓ taplo passed${NC}"
    else
        echo -e "${RED}✗ taplo failed${NC}"
        echo -e "${YELLOW}Run 'taplo format' to fix TOML formatting${NC}"
        exit 1
    fi
else
    echo -e "${YELLOW}⚠ taplo not installed, skipping TOML format check${NC}"
    echo -e "${YELLOW}Install with: cargo install taplo-cli${NC}"
fi

# Check cargo-deny (skip if deny.toml doesn't exist)
if command -v cargo-deny &> /dev/null && [[ -f "deny.toml" ]]; then
    echo "Running cargo-deny check..."
    if cargo-deny check bans licenses sources --hide-inclusion-graph --show-stats; then
        echo -e "${GREEN}✓ cargo-deny passed${NC}"
    else
        echo -e "${RED}✗ cargo-deny failed${NC}"
        exit 1
    fi
else
    echo -e "${YELLOW}⚠ cargo-deny not installed or deny.toml missing, skipping dependency check${NC}"
fi

# Run Rust tests (native)
echo -e "${YELLOW}Step 6: Running Rust tests (native targets)...${NC}"
if cargo test --workspace --exclude l3d-python -- --test-threads=1; then
    echo -e "${GREEN}✓ native tests passed${NC}\n"
else
    echo -e "${RED}✗ native tests failed${NC}\n"
    exit 1
fi

# Build WASM target (l3d-egui)
echo -e "${YELLOW}Step 7: Building WASM (Trunk)...${NC}"
if command -v trunk &> /dev/null; then
    echo "Building l3d-egui WASM with Trunk..."
    if (cd crates/l3d-egui && trunk build --release); then
        echo -e "${GREEN}✓ WASM build passed${NC}\n"
    else
        echo -e "${RED}✗ WASM build failed${NC}\n"
        exit 1
    fi
else
    echo -e "${YELLOW}⚠ trunk not installed, skipping WASM build${NC}"
    echo -e "${YELLOW}Install with:${NC}"
    echo -e "${YELLOW}  cargo install trunk${NC}"
    echo -e "${YELLOW}  rustup target add wasm32-unknown-unknown${NC}"
fi

# Run Python tests for l3d-python if tooling is available
echo -e "${YELLOW}Step 7b: Running Python tests for l3d-python...${NC}"
if command -v python3 &> /dev/null && command -v maturin &> /dev/null; then
    TMP_VENV=".venv_l3d_test"
    python3 -m venv "$TMP_VENV"
    # shellcheck disable=SC1090
    source "$TMP_VENV/bin/activate"
    python -m pip install --upgrade pip >/dev/null 2>&1

    echo "Building and installing l3d-python into venv (maturin develop)..."
    if maturin develop -m crates/l3d-python/Cargo.toml --release >/dev/null; then
        echo -e "${GREEN}✓ l3d-python built and installed${NC}"

        # Run basic import test
        if python -c "import l3d; print('l3d module loaded successfully')"; then
            echo -e "${GREEN}✓ l3d-python import test passed${NC}\n"
        else
            echo -e "${RED}✗ l3d-python import test failed${NC}\n"
            deactivate || true
            rm -rf "$TMP_VENV"
            exit 1
        fi
    else
        echo -e "${RED}✗ maturin develop failed for l3d-python${NC}"
        deactivate || true
        rm -rf "$TMP_VENV"
        exit 1
    fi

    deactivate || true
    rm -rf "$TMP_VENV"
else
    echo -e "${YELLOW}⚠ python3 or maturin not installed, skipping l3d-python tests${NC}"
    echo -e "${YELLOW}Install with:${NC}"
    echo -e "${YELLOW}  pipx install maturin  (or: pip install maturin)${NC}"
fi

# Pre-publish checks
echo -e "\n${BLUE}=== Pre-publish Checks for crates.io ===${NC}\n"

# Check package metadata
echo -e "${YELLOW}Step 8: Validating package metadata...${NC}"
for crate_dir in crates/*/; do
    crate_name=$(basename "$crate_dir")

    # Skip l3d-python (uses PyPI, not crates.io)
    if [[ "$crate_name" == "l3d-python" ]]; then
        echo -e "${YELLOW}Skipping $crate_name (published to PyPI, not crates.io)${NC}"
        continue
    fi

    echo "Checking $crate_name..."

    if (cd "$crate_dir" && cargo package --list --allow-dirty > /dev/null 2>&1); then
        echo -e "${GREEN}✓ $crate_name package metadata valid${NC}"
    else
        echo -e "${RED}✗ $crate_name package metadata invalid${NC}"
        echo -e "${YELLOW}Run 'cd $crate_dir && cargo package --list' for details${NC}"
        exit 1
    fi
done
echo ""

# Dry-run publish in dependency order
echo -e "${YELLOW}Step 9: Running dry-run publish (in dependency order)...${NC}"

# Define publish order: dependencies first, then dependents
PUBLISH_ORDER=(
    "l3d_rs"
    "l3d-ffi"
    "l3d-egui"
)

for crate_name in "${PUBLISH_ORDER[@]}"; do
    crate_dir="crates/$crate_name"

    if [[ ! -d "$crate_dir" ]]; then
        echo -e "${YELLOW}⚠ Skipping $crate_name (directory not found)${NC}"
        continue
    fi

    echo "Validating $crate_name package..."

    # For l3d_rs (no dependencies), do full dry-run publish
    if [[ "$crate_name" == "l3d_rs" ]]; then
        if (cd "$crate_dir" && cargo publish --dry-run --allow-dirty); then
            echo -e "${GREEN}✓ $crate_name dry-run publish passed${NC}"
        else
            echo -e "${RED}✗ $crate_name dry-run publish failed${NC}"
            exit 1
        fi
    else
        # For dependent crates, just verify package contents
        # (can't do full dry-run until dependencies are on crates.io)
        if (cd "$crate_dir" && cargo package --allow-dirty --list > /dev/null 2>&1); then
            echo -e "${GREEN}✓ $crate_name package validation passed${NC}"
            echo -e "${BLUE}  Note: Full publish validation will happen after l3d_rs is published${NC}"
        else
            echo -e "${RED}✗ $crate_name package validation failed${NC}"
            exit 1
        fi
    fi
done
echo ""

# Check for uncommitted changes
echo -e "${YELLOW}Step 10: Checking for uncommitted changes...${NC}"
if [[ -n $(git status --porcelain 2>/dev/null || echo "") ]]; then
    echo -e "${YELLOW}⚠ You have uncommitted changes:${NC}"
    git status --short
    echo -e "${YELLOW}Consider committing or stashing changes before publishing${NC}\n"
else
    echo -e "${GREEN}✓ No uncommitted changes${NC}\n"
fi

# Check if on main branch
echo -e "${YELLOW}Step 11: Checking git branch...${NC}"
current_branch=$(git branch --show-current 2>/dev/null || echo "unknown")
if [[ "$current_branch" != "main" && "$current_branch" != "master" ]]; then
    echo -e "${YELLOW}⚠ You are on branch '$current_branch', not 'main'${NC}"
    echo -e "${YELLOW}Consider switching to main branch before publishing${NC}\n"
else
    echo -e "${GREEN}✓ On $current_branch branch${NC}\n"
fi

echo -e "\n${GREEN}=== All CI checks passed! ===${NC}"
echo -e "${GREEN}Your code is ready to be pushed.${NC}"
echo -e "\n${BLUE}Publishing Order (IMPORTANT - follow this sequence):${NC}"
echo -e "  ${YELLOW}1.${NC} Commit and push changes"
echo -e "  ${YELLOW}2.${NC} Tag the release: git tag v0.2.1 && git push origin v0.2.1"
echo -e "  ${YELLOW}3.${NC} CI will publish to crates.io and PyPI automatically"
echo -e "  ${YELLOW}Or manual publish:${NC}"
echo -e "     ${YELLOW}cd crates/l3d_rs && cargo publish${NC}"
echo -e "     ${YELLOW}cd crates/l3d-ffi && cargo publish${NC}"
echo -e "     ${YELLOW}cd crates/l3d-egui && cargo publish${NC}"
echo -e "     ${YELLOW}cd crates/l3d-python && maturin publish${NC}"
