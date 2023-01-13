# docker build -t baidang201/noahs-ark-node .
# This is the build stage for Substrate. Here we create the binary.
FROM docker.io/paritytech/ci-linux:production as builder

WORKDIR /redstone-node
COPY . /redstone-node
RUN git submodule update --init --recursive
RUN git submodule sync --recursive
RUN git submodule update --init --recursive
RUN cargo build --locked --release

# This is the 2nd stage: a very small image where we copy the Substrate binary."
FROM docker.io/library/ubuntu:20.04
LABEL description="Multistage Docker image for Substrate: a platform for web3" \
	io.parity.image.type="builder" \
	io.parity.image.authors="chevdor@gmail.com, devops-team@parity.io" \
	io.parity.image.vendor="Parity Technologies" \
	io.parity.image.description="Substrate is a next-generation framework for blockchain innovation 🚀" \
	io.parity.image.source="https://github.com/paritytech/polkadot/blob/${VCS_REF}/docker/substrate_builder.Dockerfile" \
	io.parity.image.documentation="https://github.com/paritytech/polkadot/"

COPY --from=builder /redstone-node/target/release/redstone-node /usr/local/bin

RUN useradd -m -u 1000 -U -s /bin/sh -d /redstone-node redstone-node && \
	mkdir -p /data /redstone-node/.local/share/redstone-node && \
	chown -R redstone-node:redstone-node /data && \
	ln -s /data /redstone-node/.local/share/redstone-node && \
# unclutter and minimize the attack surface
	rm -rf /usr/bin /usr/sbin && \
# Sanity checks
	/usr/local/bin/redstone-node --version

USER redstone-node
EXPOSE 30333 9933 9944 9615
VOLUME ["/data"]