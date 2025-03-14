# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.10.1](https://github.com/starship/rust-battery/compare/v0.10.0...v0.10.1) (2025-03-14)


### Bug Fixes

* **darwin:** prefer non-raw values for state_of_charge ([#85](https://github.com/starship/rust-battery/issues/85)) ([07c79be](https://github.com/starship/rust-battery/commit/07c79beb2a1731d545ab078bfe90219a8a883984))

## [0.10.0](https://github.com/starship/rust-battery/compare/v0.9.1...v0.10.0) (2024-08-31)


### Features

* replace winapi with windows-sys ([#70](https://github.com/starship/rust-battery/issues/70)) ([7f69364](https://github.com/starship/rust-battery/commit/7f69364803b855d7abc6272c4b02b6499099c60e))

## [0.9.1](https://github.com/starship/rust-battery/compare/v0.9.0...v0.9.1) (2024-07-25)


### Bug Fixes

* remove `netbsd` keyword ([#72](https://github.com/starship/rust-battery/issues/72)) ([764972b](https://github.com/starship/rust-battery/commit/764972bd0c5145f8eb472acb06a9af1f8e626a85))

## [0.9.0](https://github.com/starship/rust-battery/compare/v0.8.3...v0.9.0) (2024-06-08)


### Features

* NetBSD support with envsys ioctl and plist ([#69](https://github.com/starship/rust-battery/issues/69)) ([87b8cc4](https://github.com/starship/rust-battery/commit/87b8cc45ec2b25922069f1dbb6bb7b27cb655e35))

## [0.8.3](https://github.com/starship/rust-battery/compare/v0.8.2...v0.8.3) (2024-04-02)


### Bug Fixes

* **linux:** be more permissive on invalid UTF-8 file content ([#60](https://github.com/starship/rust-battery/issues/60)) ([3a229ec](https://github.com/starship/rust-battery/commit/3a229ec4a240ffaed99706c315a29e3ece691e5b))

## [0.8.2](https://github.com/starship/rust-battery/compare/v0.8.1...v0.8.2) (2023-08-05)


### Bug Fixes

* **darwin:** allow fallback to non-raw key & permit errors ([#41](https://github.com/starship/rust-battery/issues/41)) ([19f12f0](https://github.com/starship/rust-battery/commit/19f12f04c1194515131b6edd7a1f155e3c954573))

## [0.8.1](https://github.com/starship/rust-battery/compare/v0.8.0...v0.8.1) (2023-06-08)


### Bug Fixes

* use correct data source for querying max capacity on arm64-macOS ([#33](https://github.com/starship/rust-battery/issues/33)) ([fadcee6](https://github.com/starship/rust-battery/commit/fadcee6c5e052ba58e92ea0290e1d582b2609e4f))

## [0.8.0](https://github.com/starship/rust-battery/compare/v0.7.9...v0.8.0) (2023-04-12)


### ⚠ BREAKING CHANGES

* rework ci & code cleanup
* rework ci & code cleanup

### Features

* derive `serde` and `schemars` traits for `State` enum ([#2](https://github.com/starship/rust-battery/issues/2)) ([487ebc2](https://github.com/starship/rust-battery/commit/487ebc2f7fbd30c346c13f82a1f721fa0256d43a))


### Bug Fixes

* bump msrv ([26bea66](https://github.com/starship/rust-battery/commit/26bea66aea58ccfd8df6005a54c403bf89554b6d))
* bump msrv ([bfcf16d](https://github.com/starship/rust-battery/commit/bfcf16db3297b05565b853e2dc19cdfd03c32986))
* bump msrv further ([#27](https://github.com/starship/rust-battery/issues/27)) ([79f1359](https://github.com/starship/rust-battery/commit/79f1359b3d4ca2f3247f04b6f72a0ca2a6db5811))
* replace `mach` with `mach2` fork ([#28](https://github.com/starship/rust-battery/issues/28)) ([ada2f1b](https://github.com/starship/rust-battery/commit/ada2f1b2ab0fd8c8c6bea7ff47e623d5fa0c94ea))


### Code Refactoring

* rework ci & code cleanup ([2330404](https://github.com/starship/rust-battery/commit/2330404d7b0f57be47f733905c735f96e866e401))
* rework ci & code cleanup ([b507347](https://github.com/starship/rust-battery/commit/b507347036b237405e292c23f7ce50dfb4ab8e58))

## [0.7.9] - 2021-11-09

- Update `nix` dependency version
- Allow empty `FullyCharged` and `DesignCapacity`

## [0.7.8] - 2020-11-01

### Fixed

- Add `nix` dependency back for FreeBSD build (#76)

## [0.7.7] - 2020-10-19

### Fixed

- Update dependencies versions to fix `nightly` toolchain compilation

## [0.7.6] - 2020-08-24

- MSRV changed to Rust `1.36.0`

### Fixed

- Zero energy rate is not considered as an error for Windows [#63](https://github.com/svartalf/rust-battery/issues/63)

## [0.7.5] - 2019-11-26
### Fixed

- Handling missing `energy_full_design` source files for Linux [#40](https://github.com/svartalf/rust-battery/issues/40)

## [0.7.4] - 2019-06-03
### Fixed
- `Manager::refresh` method in Linux implementation checks if battery folder is exists [#29](https://github.com/svartalf/rust-battery/issues/29)
- `Battery::energy_full_design` is not falling into a infinite recursion anymore [#30](https://github.com/svartalf/rust-battery/issues/30)

## [0.7.3] - 2019-05-30
### Fixed
- `ENODEV` errors for Linux are now handled the same as `ENOENT` [#28](https://github.com/svartalf/rust-battery/issues/28)

### Changed
- Relaxing `uom` dependency version to `^0.23` for `battery` crate
- Relaxing `libc` dependency version to `^0.2` for `battery-ffi` crate

## [0.7.2] - 2019-05-21
### Fixed
- `Battery::state_of_health` and `Battery::state_of_charge` are always returning values in `0.0 ≤ x ≤ 1.0` interval

## [0.7.1] - 2019-03-31
### Changed
- `uom`, `core-foundation` and `libc` dependencies were updated to latest versions
- Zero cycles count is considered as non-existing value for Linux [#23](https://github.com/svartalf/rust-battery/issues/23)
### Removed
- `battery-cli` crate was yanked and replaced with `battop` crate (https://crates.io/crates/battop)

## [0.7.0] - 2019-03-10
### Changed
- Propagate all errors happened from `battery` and `battery-ffi` crates to the caller
- Return SI measurement units from `uom` crate for almost all public `Battery` methods
- Re-export used `uom` quantities and measurement units in public `battery::units` module
- Rename `Battery::percentage` method into `Battery::state_of_charge`
- Rename `Battery::capacity` method into `Battery::state_of_health`
- Mark `battery::State` and `battery::Technology` enums as a non-exhaustive
- Support multiple devices for FreeBSD and DragonFlyBSD [#17](https://github.com/svartalf/rust-battery/issues/17)
- Ignore devices with `scope` attributes different from `System` for Linux [#18](https://github.com/svartalf/rust-battery/issues/18)
- Update outdated `mach` dependency for Mac OS

## [0.6.2] - 2019-02-28
### Changed
- Replace looks-to-be-abandoned `CoreFoundation-sys` and `IOKit-sys` dependencies [#2](https://github.com/svartalf/rust-battery/issues/2)
### Fixed
- Free hanging mach port used for communication with Mac OS IOKit

## [0.6.1] - 2019-02-27
### Fixed
- Fix energy and remaining time calculations for MacOS [#8](https://github.com/svartalf/rust-battery/issues/8), [#11](https://github.com/svartalf/rust-battery/pull/11)
- Fix multiplication overflow while calculating battery percentage in Mac OS by [@mindriot101](https://github.com/mindriot101) [#10](https://github.com/svartalf/rust-battery/pull/10)
- Fix wrong units for consumption graph in `battery-cli`, should be `W` instead of `Wh` [#9](https://github.com/svartalf/rust-battery/issues/9)
- Fix non-uniform path import that was breaking compilation for Rust<1.32 [#6](https://github.com/svartalf/rust-battery/issues/6)
- Fix `time_to_empty` and `time_to_full` calculations for Linux when charger is unplugged but driver still reports zero `energy_rate` by [@kerhong](https://github.com/kerhong) [#5](https://github.com/svartalf/rust-battery/pull/5)
