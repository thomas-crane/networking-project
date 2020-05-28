# usage: ./do-run.sh normal iot udp 1
# meaning, use normal network conditions, iot payload size, udp protocol, and
# this is run number 1.

# refresh containers
echo "Refreshing containers..."
./rm-containers.sh > /dev/null 2>&1
./create-containers.sh > /dev/null 2>&1

# netem stuff
echo "Applying netem rules"
case "$1" in
  normal)
    # normal network
    docker exec consumer tc qdisc add dev eth0 root netem delay 20ms 10ms loss random 0.5%
    docker exec producer tc qdisc add dev eth0 root netem delay 20ms 10ms loss random 0.5%
    ;;
  acceptable)
    # acceptable network
    docker exec consumer tc qdisc add dev eth0 root netem delay 40ms 20ms loss random 2%
    docker exec producer tc qdisc add dev eth0 root netem delay 40ms 20ms loss random 2%
    ;;
  degraded)
    # degraded network
    docker exec consumer tc qdisc add dev eth0 root netem delay 80ms 20ms loss random 6%
    docker exec producer tc qdisc add dev eth0 root netem delay 80ms 20ms loss random 6%
    ;;
  horrible)
    # horrible network
    docker exec consumer tc qdisc add dev eth0 root netem delay 150ms 30ms loss random 12%
    docker exec producer tc qdisc add dev eth0 root netem delay 150ms 30ms loss random 12%
    ;;
  *)
    echo "Unknown network quality $1. exiting"
    exit 1
    ;;
esac

# set payload size.
case "$2" in
  iot)
    PAYLOAD_SIZE=64
    ;;
  stream)
    PAYLOAD_SIZE=512
    ;;
  web)
    PAYLOAD_SIZE=4096
    ;;
  *)
    echo "Unknown payload size $2. exiting"
    exit 1
    ;;
esac
echo "Using payload size $PAYLOAD_SIZE"

# set protocol
case "$3" in
  tcp)
    PROTOCOL=tcp
    ;;
  udp)
    PROTOCOL=udp
    ;;
  srdp)
    PROTOCOL=srdp
    ;;
  *)
    echo "Unkown protocol $3. exiting"
    exit 1
    ;;
esac

# do the run
echo "Running"
./run.sh $PROTOCOL 1000 10 $PAYLOAD_SIZE

# move files
# folder is logs/payload_size/protocol/network_quality/runs
echo "Moving producer.txt to logs/$2/$3/$1/$4-producer.txt"
mv producer.txt ../logs/$2/$3/$1/$4-producer.txt
echo "Moving consumer to logs/$2/$3/$1/$4-consumer.txt"
mv consumer.txt ../logs/$2/$3/$1/$4-consumer.txt

# notify of completion
notify-send -u low -a Runner -t 5000 "Run $4 of $1 $2 $3 has finished."
