# HT16K33

[![Version info](https://img.shields.io/crates/v/ht16k33.svg)](https://crates.io/crates/ht16k33)
[![Documentation](https://docs.rs/ht16k33/badge.svg)](https://docs.rs/ht16k33)
[![Build Status](https://travis-ci.org/jasonpeacock/ht16k33.svg?branch=master)](https://travis-ci.org/jasonpeacock/ht16k33)
[![Average time to resolve an issue](http://isitmaintained.com/badge/resolution/jasonpeacock/ht16k33.svg)](http://isitmaintained.com/project/jasonpeacock/ht16k33 "Average time to resolve an issue")
[![Percentage of issues still open](http://isitmaintained.com/badge/open/jasonpeacock/ht16k33.svg)](http://isitmaintained.com/project/jasonpeacock/ht16k33 "Percentage of issues still open")

`ht16k33` is a driver for the [Holtek HT16K33 "RAM Mapping 16\*8 LED Controller Driver with keyscan"](http://www.holtek.com/productdetail/-/vg/HT16K33).

Currently, only the 28-pin SOP package type is supported.

## Features

- [x] Uses the [`embedded-hal`](https://crates.io/crates/embedded-hal) hardware abstraction.
- [x] Supports `no_std` for embedded devices.
- [ ] Supports all 20/24/28-pin SOP package types.
- [x] Displays all 128 LEDs.
- [ ] Reads keyscan.
- [ ] Manages interrupts.
- [ ] Manages slave devices.

## Support

For questions, issues, feature requests, and other changes, please file an [issue in the github project](https://github.com/jasonpeacock/ht16k33/issues).

## License

Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you,
as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
