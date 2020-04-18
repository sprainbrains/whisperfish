ARG SFOS_VERSION=latest

FROM coderus/sailfishos-platform-sdk:$SFOS_VERSION as coderus_base

FROM fedora:latest

# Install cross compilers

RUN dnf install -y \
    gcc-arm-linux-gnu gcc-c++-arm-linux-gnu binutils-arm-linux-gnu \
    gcc-aarch64-linux-gnu gcc-c++-aarch64-linux-gnu binutils-aarch64-linux-gnu \
    curl \
    rpm-build
RUN dnf groupinstall -y "Development Tools"

# Install MER SDK

COPY --from=coderus_base /srv/mer /srv/mer

# Install Rust
ENV RUSTUP_HOME /usr/local/rustup
ENV CARGO_HOME /usr/local/cargo
ENV PATH="$CARGO_HOME/bin:$PATH"

RUN curl --proto '=https' --tlsv1.2 -sSf -JO "https://static.rust-lang.org/rustup/dist/x86_64-unknown-linux-gnu/rustup-init"
RUN chmod +x rustup-init
RUN ./rustup-init -y --no-modify-path --default-toolchain stable
RUN rustup --version
RUN cargo --version
RUN rustc --version

# Install nightly and beta
RUN rustup toolchain install nightly
RUN rustup toolchain install beta

# Set environment
ENV MER_SDK /srv/mer

RUN ln -s $MER_SDK/targets/SailfishOS-3.3.0.14-armv7hl/* /usr/arm-linux-gnu/sys-root/

# Install cargo-rpm
RUN cargo install --git https://github.com/RustRPM/cargo-rpm --branch develop

# Add cargo targets
RUN rustup target add \
    arm-unknown-linux-gnueabihf \
    aarch64-unknown-linux-gnu

# Additional C dependencies for Whisperfish
RUN dnf install -y cmake

RUN mkdir /root/.cargo
COPY .ci/cargo.toml /root/.cargo/config
