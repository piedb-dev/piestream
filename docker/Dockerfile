FROM ubuntu:22.04 as builder

ENV LANG en_US.utf8

RUN apt-get update && DEBIAN_FRONTEND=noninteractive apt-get -y install make build-essential cmake protobuf-compiler curl pkg-config bash lld

SHELL ["/bin/bash", "-c"]

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- --no-modify-path --default-toolchain none -y

RUN mkdir -p /piestream

WORKDIR /piestream

COPY proto proto
COPY src src
COPY rust-toolchain rust-toolchain
COPY Cargo.toml Cargo.toml

COPY ./ /piestream

ENV PATH /root/.cargo/bin/:$PATH

# We need to add the `rustfmt` dependency, otherwise `piestream_pb` will not compile
RUN rustup self update \
  && rustup set profile minimal \
  && rustup default $(cat "/piestream/rust-toolchain") \
  && rustup component add rustfmt

RUN cargo fetch
RUN mkdir -p /piestream/bin
RUN cargo build -p piestream_cmd -p piestream_cmd_all --release --features static-link && \
  mv /piestream/target/release/{frontend,compute-node,meta-node,compactor,piestream} /piestream/bin/ && \
  cargo clean
RUN objcopy --compress-debug-sections=zlib-gnu /piestream/bin/piestream && \
  objcopy --compress-debug-sections=zlib-gnu /piestream/bin/frontend && \
  objcopy --compress-debug-sections=zlib-gnu /piestream/bin/compute-node && \
  objcopy --compress-debug-sections=zlib-gnu /piestream/bin/meta-node && \
  objcopy --compress-debug-sections=zlib-gnu /piestream/bin/compactor

FROM ubuntu:22.04 as image-base
RUN apt-get update && DEBIAN_FRONTEND=noninteractive apt-get -y install ca-certificates && rm -rf /var/lib/{apt,dpkg,cache,log}/

FROM image-base as frontend-node
LABEL org.opencontainers.image.source https://github.com/singularity-data/piestream
RUN mkdir -p /piestream/bin
COPY --from=builder /piestream/bin/frontend /piestream/bin/frontend
EXPOSE 4566
ENTRYPOINT [ "/piestream/bin/frontend" ]

FROM image-base as compute-node
LABEL org.opencontainers.image.source https://github.com/singularity-data/piestream
RUN mkdir -p /piestream/bin
COPY --from=builder /piestream/bin/compute-node /piestream/bin/compute-node
EXPOSE 5687
ENTRYPOINT [ "/piestream/bin/compute-node" ]

FROM image-base as meta-node
LABEL org.opencontainers.image.source https://github.com/singularity-data/piestream
RUN mkdir -p /piestream/bin
COPY --from=builder /piestream/bin/meta-node /piestream/bin/meta-node
EXPOSE 5690
ENTRYPOINT [ "/piestream/bin/meta-node" ]

FROM image-base as compactor-node
LABEL org.opencontainers.image.source https://github.com/singularity-data/piestream
RUN mkdir -p /piestream/bin
COPY --from=builder /piestream/bin/compactor /piestream/bin/compactor
EXPOSE 6660
ENTRYPOINT [ "/piestream/bin/compactor" ]

FROM image-base as piestream
LABEL org.opencontainers.image.source https://github.com/singularity-data/piestream
RUN mkdir -p /piestream/bin
COPY --from=builder /piestream/bin/piestream /piestream/bin/piestream
# Copy risedev.yml into container
COPY ./risedev.yml /risedev.yml
# Set default playground mode to docker-playground profile
ENV PLAYGROUND_PROFILE docker-playground
ENTRYPOINT [ "/piestream/bin/piestream" ]
CMD [ "playground" ]
