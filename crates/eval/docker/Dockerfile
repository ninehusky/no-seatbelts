# Install system dependencies
# Use Ubuntu as base image for better libseccomp support
FROM ubuntu:latest

# Install system dependencies.
# Importantly, this installs gcc for cross-compilation to ARM and x86.
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    llvm \
    curl build-essential pkg-config libssl-dev python3 python3-pip \
    git ca-certificates llvm file openssh-client git-lfs \
    gcc-i686-linux-gnu \
    g++-i686-linux-gnu \
    gcc-arm-linux-gnueabihf \
    g++-arm-linux-gnueabihf \
    libc6-dev-armhf-cross \
    libstdc++-13-dev-armhf-cross \
    linux-libc-dev-armhf-cross \
    docker.io \
    && rm -rf /var/lib/apt/lists/*

# Download and install xPack RISC-V GCC toolchain
RUN mkdir -p /opt/riscv && \
    curl -LO https://github.com/xpack-dev-tools/riscv-none-elf-gcc-xpack/releases/download/v14.2.0-3/xpack-riscv-none-elf-gcc-14.2.0-3-linux-x64.tar.gz && \
    tar -xzf xpack-riscv-none-elf-gcc-14.2.0-3-linux-x64.tar.gz -C /opt/riscv && \
    rm xpack-riscv-none-elf-gcc-14.2.0-3-linux-x64.tar.gz

ENV PATH="/opt/riscv/xpack-riscv-none-elf-gcc-14.2.0-3/bin:${PATH}"

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs/ | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

ENV ROOT_DIR=/app

# Set working directory
WORKDIR /app

COPY . .

# Add Rust targets for cross-compilation
RUN rustup target add i686-unknown-linux-gnu \
    && rustup target add armv7-unknown-linux-gnueabihf \
    && rustup target add riscv32imac-unknown-none-elf

# Install rustfilt for Rust symbol demangling
RUN cargo install rustfilt

# Set the default command
CMD ["bash"]