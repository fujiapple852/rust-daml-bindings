FROM ubuntu:18.04
RUN apt-get update && apt-get install -y curl git ssh build-essential openjdk-8-jre-headless
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | bash -s -- -y
RUN curl -sSL https://get.daml.com/ | sh
ENV PATH /root/.cargo/bin:/root/.daml/bin:$PATH