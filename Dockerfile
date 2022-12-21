FROM ekidd/rust-musl-builder as builder

WORKDIR /home/rust/

# Avoid having to install/build all dependencies by copying
# the Cargo files and making a dummy src/main.rs
COPY Cargo.toml .
COPY Cargo.lock .

RUN echo "fn main() {}" > src/main.rs
# RUN cargo test
RUN cargo build --release

# We need to touch our real main.rs file or else docker will use
# the cached one.
ADD --chown=rust:rust . ./

RUN touch src/main.rs

# RUN cargo test
RUN cargo build --release

# Size optimization
RUN strip target/x86_64-unknown-linux-musl/release/rust-cli-rest-perf-tests

# Start building the final image
FROM alpine
WORKDIR /home/rust/
COPY --from=builder /home/rust/target/x86_64-unknown-linux-musl/release/rust-cli-rest-perf-tests .
CMD ["./rust-cli-rest-perf-tests"]