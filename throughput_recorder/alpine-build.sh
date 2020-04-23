docker run \
  --rm \
  -u $(id -u):$(id -g) \
  -v "$PWD":/usr/src/throughput_recorder \
  -w /usr/src/throughput_recorder \
  rust:1.42.0-alpine \
  cargo build --target x86_64-unknown-linux-musl --release
