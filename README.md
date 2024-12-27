# nrf-modem test
To build, start by cloning the latest release of `embassy` and `nrf-modem` (in the same directory for relative reference)

### Build 
To build, clone into the `~/embassy/examples/nrf9151/` directory
$ cargo clean
$ cargo build --bin embassy-nrf9151-non-secure-examples --release
```

### Flash
```
cargo run --bin embassy-nrf9151-non-secure-examples
```

WIP - still need more testing ...



