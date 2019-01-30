FROM rust:latest
WORKDIR /usr/src/nihctfplat
COPY . .
RUN cargo build --release

FROM debian:stable-slim
RUN apt-get update && apt-get install -y libpq5 && rm -rf /var/lib/apt/lists/*
COPY --from=0 /usr/src/nihctfplat/target/release/nihctfplat /usr/local/bin/nihctfplat
CMD /usr/local/bin/nihctfplat

# vi:syntax=dockerfile