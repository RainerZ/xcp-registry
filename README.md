# xcp-registry

Measurement and calibration registry for XCP in pure Rust, with the derive macros that fill it.

The registry holds the description of all measurement and calibration objects of an application:
events, calibration segments, type definitions and instances. From it an A2L file is written,
which a tool such as CANape uses to access the application over XCP.

## Crates

This repository contains two crates. They are released together with the same version number.

| Crate | Content | Links |
|---|---|---|
| [`xcp_registry`](xcp_registry/README.md) | The registry: data model, A2L writer, optional A2L reader, JSON | [crates.io](https://crates.io/crates/xcp_registry) · [docs.rs](https://docs.rs/xcp_registry) |
| [`xcp_registry_derive`](xcp_registry_derive/README.md) | `#[derive(McRegisterType)]` and `#[derive(McRegisterEnum)]` | [crates.io](https://crates.io/crates/xcp_registry_derive) · [docs.rs](https://docs.rs/xcp_registry_derive) |

An application only needs `xcp_registry`, it re-exports the derive macros:

```toml
[dependencies]
xcp_registry = "0.1"
```

The feature `a2l_reader` adds loading and checking of A2L files. The documentation with examples is
in the [README of `xcp_registry`](xcp_registry/README.md).

## Related projects

- [xcp-lite](https://github.com/vectorgrp/xcp-lite): XCP for Rust. The registry was developed as part
  of xcp-lite and moved into this repository after xcp-lite V3.0.11.
- [XCPlite](https://github.com/vectorgrp/XCPlite): XCP for C and C++. Its tool `xcpclient` uses the
  registry to read and write A2L files.

## Building and testing

The minimum supported Rust version is 1.90.

```bash
cargo test --workspace --features xcp_registry/a2l_reader
```

```bash
cargo clippy --workspace --all-targets --features xcp_registry/a2l_reader
```

The A2L files in `xcp_registry/tests/fixtures/` are test cases, see the comment at the top of
[`xcp_registry/tests/round_trip.rs`](xcp_registry/tests/round_trip.rs) for how to add one.

Changes are listed in the [changelog](CHANGELOG.md).

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <https://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <https://opensource.org/licenses/MIT>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in
the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without
any additional terms or conditions.
