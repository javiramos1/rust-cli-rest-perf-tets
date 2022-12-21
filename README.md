# Rust CLI example to run HTTP performance Tests

Tiny HTTP performance test CLI written in Rust.

The goal of this project is to showcase Rust capabilities including the powerful **macros** which the [clap](https://github.com/clap-rs/clap) library uses to create easy to write CLIs.

This application can run performance tests and printout the results. It support GET and POST requests with an additional body.

**NOTE:** This is just a personal project to demo Rust capabilities. It is not intended to be used in Production.

## Goals

This project has a Makefile with the following goals:

- `make bld`: Builds the  application.
- `make run`: Runs application.
- `make build`: Builds docker image.
- `make push`: Pushes docker image.

## CLI


```
Rust CLI using clap library which send HTTP requests that can be used for performance test

Usage: rust-cli-rest-perf-tets [OPTIONS] <METHOD> <URL>

Arguments:
  <METHOD>  HTTP method
  <URL>     Target URL

Options:
  -p, --producers <PRODUCERS>
          Number of Producers sending requests [default: 1]
  -e, --expected-status <EXPECTED_STATUS>
          Expected HTTP Return Status [default: 200]
  -r, --requests <REQUESTS>
          Number of Request to send [default: 1000]
  -b, --body <BODY>
          Body for HTTP POST requests
  -t, --throttle-ms <THROTTLE_MS>
          Wait time in milliseconds between requests [default: 0]
  -m, --max-ramp-up-time <MAX_RAMP_UP_TIME>
          Ramp up delay for each producer in milliseconds, this is the maximum time, a number between zero and this one will be selected. Default is the number of producers, set 0 to disable [default: -1]
  -h, --help
          Print help information
  -V, --version
          Print version information
```