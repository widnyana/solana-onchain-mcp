#!/usr/bin/env bash
set -euo pipefail

REPO="widnyana/solana-onchain-mcp"
BINARY_NAME="solana-onchain-mcp"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"

main() {
    local os arch os_name arch_name asset_url download_dir

    # Detect OS
    os="$(uname -s)"
    case "$os" in
        Linux) os_name="linux" ;;
        Darwin) os_name="darwin" ;;
        *) die "Unsupported OS: $os" ;;
    esac

    # Detect architecture
    arch="$(uname -m)"
    case "$arch" in
        x86_64|amd64) arch_name="amd64" ;;
        aarch64|arm64) arch_name="arm64" ;;
        *) die "Unsupported architecture: $arch" ;;
    esac

    # Get latest release tag
    echo "Fetching latest release..."
    local latest_tag
    latest_tag="$(fetch_latest_tag)"
    echo "Latest release: $latest_tag"

    # Construct asset URL
    local asset_name="${BINARY_NAME}-${os_name}-${arch_name}.tar.gz"
    asset_url="https://github.com/${REPO}/releases/download/${latest_tag}/${asset_name}"
    local checksums_url="https://github.com/${REPO}/releases/download/${latest_tag}/checksums.txt"

    # Create temp directory
    download_dir="$(mktemp -d)"
    trap 'rm -rf "$download_dir"' EXIT

    # Download assets
    echo "Downloading ${asset_name}..."
    curl -fsSL "$asset_url" -o "${download_dir}/${asset_name}"
    echo "Downloading checksums.txt..."
    curl -fsSL "$checksums_url" -o "${download_dir}/checksums.txt"

    # Verify checksum
    echo "Verifying checksum..."
    verify_checksum "$download_dir" "$asset_name"

    # Extract and install
    echo "Extracting..."
    tar -xzf "${download_dir}/${asset_name}" -C "$download_dir"

    # Create install directory if needed
    if [ ! -d "$INSTALL_DIR" ]; then
        echo "Creating directory: $INSTALL_DIR"
        mkdir -p "$INSTALL_DIR"
    fi

    # Install binary
    echo "Installing ${BINARY_NAME} to ${INSTALL_DIR}..."
    cp "${download_dir}/${BINARY_NAME}" "$INSTALL_DIR/"
    chmod +x "${INSTALL_DIR}/${BINARY_NAME}"

    echo ""
    echo "Installation complete!"
    echo "Binary installed to: ${INSTALL_DIR}/${BINARY_NAME}"
    echo ""
    echo "Make sure ${INSTALL_DIR} is in your PATH, then run:"
    echo "  ${BINARY_NAME} --help"
}

fetch_latest_tag() {
    local url="https://api.github.com/repos/${REPO}/releases/latest"
    local tag
    tag="$(curl -fsSL "$url" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1')"
    if [ -z "$tag" ]; then
        die "Failed to fetch latest release tag"
    fi
    echo "$tag"
}

verify_checksum() {
    local dir="$1"
    local asset="$2"

    cd "$dir"

    # Determine checksum command based on OS
    if command -v sha256sum &> /dev/null; then
        local computed
        computed="$(sha256sum "$asset" | cut -d ' ' -f1)"
        local expected
        expected="$(grep "$asset" checksums.txt | cut -d ' ' -f1)"
        if [ "$computed" != "$expected" ]; then
            die "Checksum mismatch!
computed: $computed
expected: $expected"
        fi
    elif command -v shasum &> /dev/null; then
        local computed
        computed="$(shasum -a 256 "$asset" | cut -d ' ' -f1)"
        local expected
        expected="$(grep "$asset" checksums.txt | cut -d ' ' -f1)"
        if [ "$computed" != "$expected" ]; then
            die "Checksum mismatch!
computed: $computed
expected: $expected"
        fi
    else
        echo "Warning: No checksum tool found, skipping verification"
    fi

    echo "Checksum verified!"
}

die() {
    echo "Error: $*" >&2
    exit 1
}

main "$@"
