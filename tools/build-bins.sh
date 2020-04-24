if [[ ! $(basename $PWD) == networking-project ]]; then
  echo "build-bins must be run from root dir."
  exit 1
fi

projects=(
  throughput_recorder
  proxy_server
  traffic_producer
  traffic_consumer
)

mkdir -p bins/
for project in "${projects[@]}"; do
  pushd "$project"
  ../tools/alpine-build.sh
  cp target/x86_64-unknown-linux-musl/release/$project ../bins/
  popd
done
