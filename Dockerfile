FROM rust:latest

RUN apt-get update && apt-get -y install gnupg2

# Debian doesn't believe in letting you easily install latest node+npm so here we go
RUN curl -sL https://deb.nodesource.com/setup_8.x | bash - && \
    apt-get install -y nodejs
RUN npm install -g elm@0.18 --unsafe-perm=true 

COPY client client
COPY server server

RUN cd client && ./build.sh
RUN cd ../server

WORKDIR /server

RUN cargo build
# FIXME: this cannot be killed with ⌃+C, you need to `docker ps` and `docker kill` the running container
CMD cargo run
