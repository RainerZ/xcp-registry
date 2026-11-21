# xcp_registry_derive

Proc-macro crate providing `#[derive(McRegisterType)]` and `#[derive(McRegisterEnum)]`.  

The derive generates an `impl xcp_registry::McRegisterType for T` whose `register` method
calls the `xcp_registry` API directly (`add_typedef` / `add_typedef_field` / `add_instance`),
without any intermediate descriptor data structures.  

The generated code emits fully-qualified `::xcp_registry::...` paths, so the consuming crate
must list `xcp_registry` as a **direct dependency** in its `Cargo.toml`, also when it gets the
macros through a re-export of another crate such as `xcp_lite`. This crate itself does **not**
depend on `xcp_registry` (no dependency cycle), it only emits paths as tokens.  

See the design specification in [`DESIGN.md`](DESIGN.md) and examples in the `examples` folder.  

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <https://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <https://opensource.org/licenses/MIT>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in
the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without
any additional terms or conditions.
