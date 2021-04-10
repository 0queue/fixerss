FROM rust:1.51 as builder-tools
RUN apt-get update && apt-get install -y musl-tools
RUN rustup target add x86_64-unknown-linux-musl
RUN cargo install sqlx-cli

FROM builder-tools as builder
WORKDIR /usr/src/
COPY . .
# todo musl will not be an implicit statically linked target in the future
RUN mkdir target && cargo sqlx database create && cargo sqlx migrate --source server/migrations run
RUN cargo build --release --target x86_64-unknown-linux-musl --bin server

FROM scratch
WORKDIR /
# until rocket supports SIGTERM I guess
STOPSIGNAL SIGINT
ENV FIXERSS_LOG_LEVEL=normal
COPY --from=builder /usr/src/target/x86_64-unknown-linux-musl/release/server /fixerss-server
CMD ["/fixerss-server"]