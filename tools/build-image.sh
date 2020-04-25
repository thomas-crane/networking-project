if [[ ! $(basename $PWD) == networking-project ]]; then
  echo "build-image must be run from root dir."
  exit 1
fi

# build the binaries first.
echo "Building binaries..."
tools/build-bins.sh

# create the docker image.
echo "Building docker image..."
docker build . -t tom/net-project

