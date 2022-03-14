# creditcoin-loadtest
Basic loadtesting for the Creditcoin blockchain. Built around the [goose](https://docs.rs/goose) load testing library.

## Building
 Organized as a standard Rust project, so assuming you have Rust [installed](https://rustup.rs/), all that's required is to run
 
 ```
 cargo build
 ```
 ## Running
 To see the full list of command line arguments, you can run
 ```
 cargo run --release -- -h
 ```
 
 A basic loadtest targeting the QA network with 300 users spawned at a rate of 10 users/second:
 (Note: the `--host` parameter is ignored currently, as we use websocket requests instead of HTTP requests)
 ```
 cargo run --release -- --host 'https://qa.creditcoin.network' --report-file=report.html --users 300 --hatch-rate 10
 ```
 
