FROM ubuntu:22.04

WORKDIR /home/

COPY . .

RUN bash ./setup.sh
RUN bash ./node.sh

ENV PATH="/root/.cargo/bin:$PATH"

RUN curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh