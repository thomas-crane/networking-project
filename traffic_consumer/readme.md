# Traffic consumer

## About

+ This is a small tool written in Rust which can be used as a sink for network traffic.
+ It supports both TCP and UDP.
+ I'm using it to help collect data for a research project I'm currently doing.
+ Traffic is always consumed from port `6860`.

## Usage

```bash
$ cargo run -- MODE
```

Where

+ `MODE` is either `tcp` or `udp`.
