# Proxy server

## About

+ This is a small tool written in Rust which can be used as a "pass through" proxy server which simply forwards all of the traffic it receives to the appropriate destination.
+ It supports both TCP and UDP.
+ I'm using it to help collect data for a research project I'm currently doing.
+ The proxy server will always listen for traffic on port `6850`.
+ The server will always forward traffic to the port `6860`.

## Usage

```bash
$ cargo run -- MODE
```

Where

+ `MODE` is either `tcp` or `udp`

The tool also expects that there will be two environment variables present.

+ `PRODUCER_IP` - the IP address of the producer.
+ `CONSUMER_IP` - the IP address of the consumer.

### UDP mode

In UDP mode, when a message is received from the `PRODUCER_IP`, it is forwarded to the `CONSUMER_IP`, and vice versa.

### TCP mode

In TCP mode, when a connection with the TCP server is established, the proxy server will establish another connection with the appropriate remote address. Once the remote socket is established, traffic from both sockets will be forwarded to the appropriate destination.
