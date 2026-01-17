# Multi-stage Dockerfile for Mozart
# Stage 1: Build WASM from Rust
# Stage 2: Build React app
# Stage 3: Serve with nginx

# ============================================
# Stage 1: Build WASM
# ============================================
FROM rust:1.83-slim AS wasm-builder

# Install wasm-pack and dependencies
RUN apt-get update && apt-get install -y \
    curl \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

RUN curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# Add WASM target
RUN rustup target add wasm32-unknown-unknown

WORKDIR /app

# Copy Cargo files first for better caching
COPY Cargo.toml Cargo.lock ./
COPY crates/mozart-core/Cargo.toml ./crates/mozart-core/

# Create dummy source to build dependencies
RUN mkdir -p crates/mozart-core/src && \
    echo "pub fn dummy() {}" > crates/mozart-core/src/lib.rs

# Build dependencies (this layer gets cached)
RUN cargo build --release -p mozart-core --target wasm32-unknown-unknown || true

# Copy actual source
COPY crates/mozart-core/src ./crates/mozart-core/src

# Build WASM (output goes to crates/mozart-core/pkg by default)
WORKDIR /app/crates/mozart-core
RUN wasm-pack build --target web --features wasm

# ============================================
# Stage 2: Build React App
# ============================================
FROM node:20-slim AS web-builder

WORKDIR /app

# Copy package files
COPY web/package.json web/package-lock.json* ./

# Install dependencies
RUN npm install

# Copy web source
COPY web/ ./

# Copy WASM build from previous stage
COPY --from=wasm-builder /app/crates/mozart-core/pkg ./src/wasm/pkg

# Build the app
RUN npm run build

# ============================================
# Stage 3: Serve with nginx
# ============================================
FROM nginx:alpine

# Copy built files
COPY --from=web-builder /app/dist /usr/share/nginx/html

# Copy nginx config
COPY nginx.conf /etc/nginx/conf.d/default.conf

EXPOSE 80

CMD ["nginx", "-g", "daemon off;"]
