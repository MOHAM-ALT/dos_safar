#/bin/bash
# DOS Safar Build Script
echo "Building DOS Safar for ARM devices..."

# Add ARM targets
rustup target add armv7-unknown-linux-gnueabihf
rustup target add aarch64-unknown-linux-gnu

# Build for ARM32
echo "Building for ARM32 (Raspberry Pi 3^)..."
cargo build --release --target armv7-unknown-linux-gnueabihf

# Build for ARM64
echo "Building for ARM64 (Raspberry Pi 4^)..."
cargo build --release --target aarch64-unknown-linux-gnu

echo "Build completed"
echo "Binaries located in target/ directory"
