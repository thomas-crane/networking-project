# Traffic producer

## About

+ This is a small tool written in Rust which can be used to produce network traffic.
+ It supports both TCP and UDP.
+ I'm using it to help collect data for a research project I'm currently doing.
+ Traffic is always sent to port `6850`.

## Usage

```bash
$ cargo run -- MODE COUNT RATE PAYLOAD_SIZE LOG
```

Where

+ `MODE` is either `tcp` or `udp`.
+ `COUNT` is the number of packets to send.
+ `RATE` is the rate in packets/second at which to send packets.
+ `PAYLOAD_SIZE` is the size in bytes of the packet payload.
+ `LOG` is the name of the file to write the logs to.

The tool also expects that there will be some environment variables present.

+ `PROXY_IP` - the IP address of the proxy server which traffic will be sent to.

The log file will end up containing two entries, one when the tool is started and one when the tool has ended.

The format is CSV, the columns are as follows

+ Timestamp - the unix timestamp at which the log entry was made.
+ Count - the number of packets that will be produced.
+ Rate - the rate in packets/second at which the packets will be produced.
+ Payload size - the size in bytes of the packet payload.

## Example

```bash
# Generate 10 UDP packets at 5 packets/second where each packet contains 100 bytes. Log this to log.txt
# Just use localhost as the proxy. If you want to see the packets being received
# you could run `nc -l -u -p 6850 localhost` before running the producer.
$ PROXY_IP=127.0.0.1 cargo run -- udp 10 5 100 log.txt
Sent packet 1 of 10
Sent packet 2 of 10
Sent packet 3 of 10
Sent packet 4 of 10
Sent packet 5 of 10
Sent packet 6 of 10
Sent packet 7 of 10
Sent packet 8 of 10
Sent packet 9 of 10
Sent packet 10 of 10

# inspect the logs.
# as expected, the log entries are 2 seconds apart (10 packets at 5 packets/sec).
$ cat log.txt
1587702205,10,5,100
1587702207,10,5,100
```
