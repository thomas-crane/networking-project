if [[ ! $(basename $PWD) == "tools" ]]; then
  echo "run.sh must be run from the tools directory"
  exit 1
fi

if [[ ! $# == 4 ]]; then
  echo "Usage: MODE COUNT RATE PAYLOAD_SIZE"
  echo "  E.g: tcp  100   10   10000"
  exit 1
fi

# set the IPs.
source `realpath ./set-ips.sh`

# start the consumer (in the background).
docker exec -e RUST_LOG=debug consumer traffic_consumer $1 > consumer.txt &
# allow it to start for a second.
sleep 1

# start the producer.
docker exec -e RUST_LOG=debug -e CONSUMER_IP producer traffic_producer $1 $2 $3 $4 > producer.txt

# wait for the consumer to shut down before exiting.
wait
