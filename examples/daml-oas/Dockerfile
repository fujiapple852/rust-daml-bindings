FROM ubuntu:18.04
RUN apt-get update
RUN apt-get install -y curl git ssh build-essential musl-tools
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | bash -s -- -y
ENV PATH /root/.cargo/bin:$PATH
RUN rustup target add x86_64-unknown-linux-musl
COPY entrypoint.sh .
CMD /entrypoint.sh
