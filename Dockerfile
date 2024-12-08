FROM rust:latest AS chef
RUN cargo install cargo-chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim AS runtime
WORKDIR /app
RUN apt-get update && apt-get install -y --no-install-recommends \
  nodejs npm python3 python3-pip curl && \
  curl -LsSf https://astral.sh/uv/install.sh | sh && \
  apt-get clean && rm -rf /var/lib/apt/lists/* /root/.npm /root/.cache
COPY --from=builder /app/target/release/mcp-server-runner /usr/local/bin/
CMD ["mcp-server-runner"]
