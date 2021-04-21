#FROM rust
#COPY ./runtime /root
#COPY ./proto /root/compute
#COPY ./src /root/compute
#COPY ./build.rs /root/compute
#COPY ./Cargo.lock /root/compute
#COPY ./Cargo.toml /root/compute
#
#WORKDIR /root/compute
#
#RUN ls /root
#RUN ls /root/compute
#RUN rustup toolchain install nightly-2020-12-14
#RUN rustup default nightly-2020-12-14-x86_64-unknown-linux-gnu
#
#RUN cargo build --release


FROM python
ADD daemon.py /root
RUN chmod -R 777 /tmp
RUN chmod -R 777 /root
COPY ./target/release/bench /bin
COPY ./target/release/batch /bin
COPY ./target/release/redisclient /bin
COPY redis.conf /etc/redis/redis.conf
CMD [ "python", "/root/daemon.py"]
