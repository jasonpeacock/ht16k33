# HT16K33

[![Version info](https://img.shields.io/crates/v/ht16k33.svg)](https://crates.io/crates/ht16k33)
[![Documentation](https://docs.rs/ht16k33/badge.svg)](https://docs.rs/ht16k33)
[![Build Status](https://travis-ci.org/jasonpeacock/ht16k33.svg?branch=master)](https://travis-ci.org/jasonpeacock/ht16k33)
[![Average time to resolve an issue](http://isitmaintained.com/badge/resolution/jasonpeacock/ht16k33.svg)](http://isitmaintained.com/project/jasonpeacock/ht16k33 "Average time to resolve an issue")
[![Percentage of issues still open](http://isitmaintained.com/badge/open/jasonpeacock/ht16k33.svg)](http://isitmaintained.com/project/jasonpeacock/ht16k33 "Percentage of issues still open")

Rust driver for the [Holtek HT16K33 "RAM Mapping 16\*8 LED Controller Driver with keyscan"](http://www.holtek.com/productdetail/-/vg/HT16K33).

# Features

- [x] Implements the [`embedded-hal`](https://crates.io/crates/embedded-hal) Interface
- [x] Displays LEDs
- [ ] Reads Keyscan
- [ ] Manages Interrupts
- [ ] Manages Slave Devices

# Support

For questions, issues, feature requests, and other changes, please file an [issue in the github project](https://github.com/jasonpeacock/ht16k33/issues).

## Rust Versions

See the top of the [Travis configuration file](.travis.yml) for the oldest, and other, supported Rust versions.

## Platforms

* Linux
    * 32 & 64bit
    * gnu & musl
* OSX
    * 64bit

# Releasing new crates

1. Run `cargo update` to refresh dependencies.
1. Run `cargo outdated` and fix any advice.
1. Run `touch src/lib.rs && cargo clippy` and fix any advice.
1. Run `cargo clean && cargo build && cargo test` to double-check everything is happy.
1. Update the version info in `Cargo.toml` as appropriate.
1. Dry-run the publish: `cargo publish --dry-run --allow-dirty`
1. Git push the change, wait for CI to pass.
1. Tag the commit & push it: `git push vX.Y.Z; git push --tags`
1. Publish the crate: `cargo publish`

# License

Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you,
as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
