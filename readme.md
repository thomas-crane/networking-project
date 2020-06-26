# Networking project

*"Specialised protocols for use in real time applications: Adding reliability to unreliable protocols"*

[![DOI](https://zenodo.org/badge/257837921.svg)](https://zenodo.org/badge/latestdoi/257837921)

## About

This repository contains all of the code that I wrote for a university project in which I wrote a research paper. The
topic of the paper was the design, implementation, and analysis of a highly specialised protocol for usage in
applications which have a requirement for optionally reliable data transfer with low overhead.

The research consisted of the following activities, which this code helped to facilitate:

+ Run some tests to see how well TCP and UDP under various network conditions in terms of both overhead and data loss.
+ Design and implement a protocol to solve the presented problem.
+ Run the same tests against the new protocol and see how it compares to TCP and UDP to determine whether or not the
  project was a success.

In summary, the project was very successful. The new protocol (SRDP) is able to provide optionally reliable data
transfer with the smallest amount of overhead possible (1 byte). When subject to the same tests as TCP and UDP were, it
had less than 4% more overhead than UDP on average, and was able to reliably deliver all of the required packets.

## Project structure

The code for this project falls into one of three folders in this repo.

+ analysis/
+ tools/
+ crates/

### Analysis

The analysis code is a small TypeScript utility which was used to interpret the data which was collected. The tool
produces graphs using Chart.js which display various metrics about the protocols being tested over a variety of network
conditions.

### Crates

The crates folder contains the bulk of the code. It contains 4 Rust crates which are described as follows.

#### Common

Common functionality such as timestamp utils and logging.

#### Protocol

A reference implementation of the SRDP protocol. This implementation tries to have a similar API to that of Rust's
`std::net::UdpSocket`, although it is not exactly the same. This implementation is not optimal as it uses polling in
some cases where events would be better. Nonetheless, it is a correct implementation of the protocol as described by the
paper.

#### Throughput recorder

A library which can be used to take "snapshots" of a network interface. This is used for collecting data about
throughout on a network interface.

#### Traffic consumer

A sink for network traffic of various different protocols. This can be used to simply listen for traffic and record
events when traffic is received.

#### Traffic producer

A source for network traffic of various different protocols. This can be used to produce a given number of packets at a
given rate, each of a given payload size. Events are recorded when traffic is produced.

### Tools

The tools folder contains a whole bunch of bash scripts which were used to automate various tasks throughout the
project. Some of the tasks included building docker images, and automatically running the data collection code.

