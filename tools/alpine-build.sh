PROJECT_NAME=$(basename $PWD)
docker run \
  --rm \
  -u $(id -u):$(id -g) \
  -v "$PWD":/usr/src/$PROJECT_NAME \
  -w /usr/src/$PROJECT_NAME \
  rust:1.42.0-alpine \
  cargo build --target x86_64-unknown-linux-musl --release
