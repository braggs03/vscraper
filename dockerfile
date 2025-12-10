# Step 1: Build the app in a Rust environment
FROM rust:1.68 as builder

# Install necessary dependencies
RUN apt-get update && apt-get install -y \
  libwebkit2gtk-4.0-dev \
  libssl-dev \
  pkg-config \
  clang \
  libclang-dev \
  cmake \
  libgtk-3-dev \
  libappindicator3-dev \
  libdbus-1-dev \
  libgdk-pixbuf2.0-dev \
  libsecret-1-dev \
  libx11-dev \
  libxcb-xfixes0-dev \
  libxss-dev \
  libxtst-dev \
  libasound2-dev \
  libpulse-dev \
  libudev-dev \
  libxcb1-dev \
  libxcb-xinerama0-dev \
  libxt-dev \
  libx11-xcb-dev

# Set working directory for the app
WORKDIR /app

# Copy your app’s code into the Docker image
COPY . .

# Build the app in a Rust environment
RUN cargo install tauri-bundler
RUN yarn install
RUN yarn tauri build

# Step 2: Set up the final image
FROM debian:bullseye-slim

# Install dependencies for running the Tauri app
RUN apt-get update && apt-get install -y \
  libwebkit2gtk-4.0-37 \
  libssl1.1 \
  libgtk-3-0 \
  libappindicator3-1 \
  libdbus-1-3 \
  libgdk-pixbuf2.0-0 \
  libsecret-1-0 \
  libx11-6 \
  libxcb1 \
  libpulse0 \
  libasound2 \
  libudev1 \
  libxcb-xfixes0 \
  libxss1 \
  libxtst6 \
  libx11-xcb1

# Copy the compiled Tauri app from the builder image
COPY --from=builder /app/src-tauri/target/release/bundle /app

# Set the working directory
WORKDIR /app

# Set entrypoint to your Tauri app's executable
ENTRYPOINT ["./YourTauriAppExecutable"]

# Optional: Expose a port (if needed for communication with the app)
EXPOSE 8080
