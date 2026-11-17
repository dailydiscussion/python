# Multi-stage production build for Hugging Face Spaces / Render / Railway / Cloud
FROM rust:1.80-alpine AS builder

RUN apk add --no-cache musl-dev pkgconfig

WORKDIR /app
COPY Cargo.toml ./

# Cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release && rm -rf src

# Copy real sources and compile optimized binary
COPY src ./src
RUN touch src/main.rs && cargo build --release

# Minimal runtime stage (~18MB)
FROM alpine:3.20

RUN apk add --no-cache ca-certificates tzdata && \
    adduser -D -u 1000 user

USER user
ENV HOME=/home/user \
    PORT=7860 \
    PROXY_URL=http://lpmrbkqk:cpege5k4lr07@45.38.107.97:6014

WORKDIR $HOME/app
COPY --from=builder /app/target/release/moviebox-api-rs $HOME/app/moviebox-api

EXPOSE 7860

CMD ["/home/user/app/moviebox-api"]
