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

# start the proxy and the consumer.
docker exec -e PRODUCER_IP -e CONSUMER_IP -d proxy proxy_server $1
docker exec -d consumer traffic_consumer $1
# allow them to start for a second.
sleep 1

# start the producer, don't exec in detached mode (-d) because we need to wait for it to finish.
docker exec -e PROXY_IP producer traffic_producer $1 $2 $3 $4
