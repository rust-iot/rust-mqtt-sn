# Rust MQTT-SN [![crates.io](https://img.shields.io/crates/v/mqtt-sn.svg)](https://crates.io/crates/mqtt-sn) [![Documentation](https://docs.rs/mqtt-sn/badge.svg)](https://docs.rs/mqtt-sn) [![Cargo Test](https://github.com/henrikssn/rust-mqtt-sn/actions/workflows/run-test.yml/badge.svg)](https://github.com/henrikssn/rust-mqtt-sn/actions/workflows/run-test.yml)

## Introduction

Partial [Rust] implementation of the [MQTT-SN] standard, which defines the operation of MQTT optimized for sensor networks. This crate is in early development but still implements most of the [MQTT-SN] protocol.

[Rust]: https://www.rust-lang.org/
[MQTT-SN]: https://www.oasis-open.org/committees/download.php/66091/MQTT-SN_spec_v1.2.pdf


## Usage

Use Cargo to add this library as a dependency to your project. Add the following to you `Cargo.toml`:
``` toml
[dependencies]
mqtt-sn = "0.1"
```

For more information, please refer to the [API Reference].

[API Reference]: https://docs.rs/mqtt-sn

## Changelog

### 0.2.0

- Add support for `defmt` (behind the `defmt-impl` feature).


## License

This project is open source software, licensed under the terms of the [Mozilla Public License].

See [LICENSE] for full details.

[Mozilla Public License]: https://de.wikipedia.org/wiki/Mozilla_Public_License
[LICENSE]: https://github.com/henrikssn/rust-mqtt-sn/blob/master/LICENSE
