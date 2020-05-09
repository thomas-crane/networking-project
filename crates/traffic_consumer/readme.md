# Traffic consumer

## About

+ This is a small tool written in Rust which can be used as a sink for network traffic.
+ It supports both TCP and UDP.
+ I'm using it to help collect data for a research project I'm currently doing.
+ Traffic is always consumed from port `6860`.

## Usage

```bash
$ cargo run -- MODE LOG_FILE
```

Where

+ `MODE` is either `tcp` or `udp`.
+ `LOG_FILE` is the name of the log file to write to.

When the consumer receives some traffic, it will add a new entry to the log file. The log file is in CSV format and contains two columns.

+ Timestamp - the unix timestamp at which the log entry was made.
+ Bytes received - the number of bytes which were received in the packet that caused the log entry to be made.
