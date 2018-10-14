FROM rust:latest

RUN apt-get update

# Debian doesn't believe in letting you easily install latest node+npm so here we go
RUN curl -sL https://deb.nodesource.com/setup_8.x | bash - && \
    apt-get install -y nodejs
RUN npm install -g elm@0.18 --unsafe-perm=true 

# Create an empty project we'll use just for building and caching our dependencies 
RUN USER=root cargo new --bin vkm/server
WORKDIR /vkm

# Copy over the manifests
COPY server/Cargo.lock server/Cargo.lock
COPY server/Cargo.toml server/Cargo.toml

# Build and cache the dependencies
RUN cd server && cargo build
RUN rm server/src/*.rs

# Copy & build server code
COPY server/src server/src
RUN cd server && cargo clean
RUN cd server && cargo build

# Copy and build client code
COPY client client
RUN cd client && ./build.sh

# FIXME: this cannot be killed with ⌃+C for some reason
CMD cd server && cargo run
