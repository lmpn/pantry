FROM lukemathwalker/cargo-chef:latest as chef
WORKDIR /app

FROM chef AS planner
COPY ./Cargo.toml ./Cargo.lock ./
COPY ./src ./src
COPY .sqlx .sqlx
COPY templates templates
COPY migrations migrations
RUN cargo chef prepare

FROM chef AS builder
COPY --from=planner /app/recipe.json .
RUN cargo chef cook --release
COPY . .
RUN cargo build --release
RUN mv ./target/release/pantry ./pantry

FROM debian:stable-slim AS runtime
WORKDIR /app
COPY --from=builder /app/pantry /usr/local/bin/
ENTRYPOINT ["/usr/local/bin/pantry"]
