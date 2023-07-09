FROM ubuntu:18.04

WORKDIR /home/

COPY . .

RUN bash ./setup.sh
RUN bash ./node.sh
RUN curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

ENV PATH="/root/.cargo/bin:$PATH"
