FROM rustlang/rust:nightly as builder

WORKDIR /steam_gid_api
COPY . .
RUN cargo build --release --bin server

FROM debian:unstable-slim

COPY --from=builder /steam_gid_api/target/release/server /server
CMD ["/server"]
