# syntax=docker/dockerfile:1.3-labs
FROM rust:1.79.0-bullseye AS build

ENV DEBIAN_FRONTEND noninteractive

RUN curl -fsSL https://deb.nodesource.com/setup_20.x | bash - \
    && apt-get update \
    && apt-get install -y ca-certificates-java nodejs openjdk-17-jdk \
    && rm -rf /var/lib/apt/lists/* \
    && npm install -g yarn \
    && cargo install just

COPY . /opt/omnitron

RUN cd /opt/omnitron \
    && just yarn --network-timeout 1000000000 \
    && just openapi \
    && just yarn build \
    && cargo build --features mysql,postgres --release

FROM debian:bullseye-20221024
LABEL maintainer=heywoodlh

ENV DEBIAN_FRONTEND noninteractive
RUN <<EOF
  set -xe
  apt-get -y update -qq
  apt-get install --no-install-recommends -y \
    ca-certificates
  apt clean
EOF

COPY --from=build /opt/omnitron/target/release/omnitron /usr/local/bin/omnitron

VOLUME /data

ENV DOCKER 1

ENTRYPOINT ["omnitron", "--config", "/data/omnitron.yaml"]
CMD ["run"]
