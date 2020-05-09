# Throughput recorder

## About

+ This is a small tool written in Rust which can be used to record the throughput of a given network interface.
+ It's basically a fancy `cat /proc/net/dev` wrapper, so it will only work on Linux.
+ I'm using it to help collect data for a research project I'm currently doing.

## Usage

```bash
$ cargo run -- IF_NAME LOG_FILE [DELAY]
```

Where

+ `IF_NAME` is the name of the interface you want to monitor (e.g. `eth0`).
+ `LOG_FILE` is a path to the file you want to save the output to.
+ `DELAY` is an optional delay in seconds to wait between taking snapshots of the throughput. This defaults to 1 second.

For example to monitor the `eth0` interface, save the logs to `log.txt` and grab a snapshot every 2 seconds, use:

```bash
$ cargo run -- eth0 log.txt 2
```

The tool takes "snapshots" of the given interface's throughput. Each snapshot includes

+ The unix epoch time in seconds.
+ The name of the interface.
+ The total bytes received for the interface.
+ The total bytes transmitted for the interface.

For example,

```bash
$ cargo run -- eth0 log.txt 2
# run the program for a few seconds,
# `ping google.com` in the background to generate some traffic.

$ cat log.txt
1587541501,eth0,1016,0
1587541503,eth0,1016,0
1587541505,eth0,1016,0
1587541507,eth0,1438,378
1587541509,eth0,1634,574
1587541511,eth0,1872,812
1587541513,eth0,1872,812
```
