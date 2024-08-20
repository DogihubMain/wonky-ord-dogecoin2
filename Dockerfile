FROM debian:latest as builder
RUN apt-get update && \
    apt-get install -y \
    ca-certificates curl file git build-essential libssl-dev pkg-config

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs --output rustup-init.sh
RUN sh ./rustup-init.sh -y
ENV PATH=/root/.cargo/bin:$PATH
WORKDIR /usr/src/wonky-ord-dogecoin
COPY . .
RUN cargo build --release

FROM debian:latest
COPY --from=builder /usr/src/wonky-ord-dogecoin/target/release/ord /usr/local/bin/ord
WORKDIR /data
RUN apt-get update && \
    apt-get install -y \
    curl
EXPOSE 80
CMD ["ord", "server"]