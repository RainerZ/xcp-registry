# Changelog

All notable changes to `xcp_registry` and `xcp_registry_derive` are documented in this file.
Both crates are released together with the same version number.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/). The crates follow
[Semantic Versioning](https://semver.org/spec/v2.0.0.html); as long as the version is 0.x, a new
minor version may contain breaking changes.

## [0.1.0] - not yet released

First release as separate crates. Both crates were developed as part of
[xcp-lite](https://github.com/vectorgrp/xcp-lite) and are taken from xcp-lite V3.0.11. For the
history before that, see the
[changelog of xcp-lite](https://github.com/vectorgrp/xcp-lite/blob/master/CHANGELOG.md).

The changes below are relative to the registry in xcp-lite V3.0.11.

### Changed

- The derive crate is renamed from `xcp_register_type_derive` to `xcp_registry_derive`. The macros
  `McRegisterType` and `McRegisterEnum` keep their names and are still re-exported by `xcp_registry`.
- `get_lock()` returns the new type `RegistryGuard` instead of a guard type of `parking_lot`. The
  registry uses `std::sync::Mutex`, `parking_lot` is no longer a dependency. A poisoned lock is ignored.
- `load_a2l` and `check_a2l` return `RegistryError` instead of `String`.
- All public enums and all public structs with public fields are `#[non_exhaustive]`.
- A2L reader: content which is not supported or out of range is skipped with a warning instead of
  panicking.
- A2L writer: `write_a2l` returns an error instead of panicking when the registry contains something
  which can not be written to an A2L file.
- `McIdentifier`: invalid characters in a name given as `&'static str` are replaced by `_` with a
  warning instead of panicking.
- `McSupportData::get_object_type` no longer panics for `McObjectType::Unspecified`, and
  `McInstanceList::add_instance` returns an error for it.

### Added

- `RegistryGuard`, the guard returned by `get_lock()`.
- `RegistryError::A2l` and `RegistryError::ObjectTypeUnspecified`.
- `McAddrMode` is exported.
- Documentation for all public items. The examples in the README are compiled by `cargo test`.
- Tests: JSON round trip, and A2L round trips for all files in `xcp_registry/tests/fixtures/`.

### Removed

- `update_cal_seg_mapping` and `update_event_mapping`, which were not finished.
- `McAddress::set_raw_a2l_addr`, which was not used.
- `load_a2lfile` is no longer public, types of the crate `a2lfile` are not part of the API any more.
- Unsafe code. Both crates are `#![forbid(unsafe_code)]`.
