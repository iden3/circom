# Simple Dockerfile for circom 2.x
#
# Build via: docker build -t circom .
# Use via: docker run -it -v $PWD:/data circom circom mycircuit.circom

FROM rust:latest

RUN apt-get update \
  && apt-get install -y cmake build-essential \
  && rm -rf /var/lib/apt/lists/*

ADD . /circom
WORKDIR /circom
RUN RUSTFLAGS="-g" cargo build --release
RUN cargo install --path circom
RUN strip -g /usr/local/cargo/bin/circom \
  && echo "CARGO_VERSION='$(cargo --version)'" >> /etc/image-info \
  && echo "RUST_VERSION='$(rustc --version)'" >> /etc/image-info

VOLUME /data
WORKDIR /data

CMD /usr/local/cargo/bin/circom
