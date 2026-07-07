FROM rust:bookworm AS builder

WORKDIR /usr/src/app

RUN apt-get update && apt-get install -y pkg-config libssl-dev
# Since the workspace is large, we should copy everything or rely on context
COPY . .

RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y openssl ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/src/app/target/release/slack-backend /usr/local/bin/slack-backend

ENV PORT=8080
EXPOSE 8080

CMD ["slack-backend"]
