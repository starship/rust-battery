# battery

[![Latest Version](https://img.shields.io/crates/v/starship-battery.svg)](https://crates.io/crates/starship-battery)
[![Latest Version](https://docs.rs/starship-battery/badge.svg)](https://docs.rs/starship-battery)
[![Build Status](https://github.com/starship/rust-battery/workflows/Continuous%20integration/badge.svg)](https://github.com/starship/rust-battery/actions?workflow=Continuous+integration)
![Minimum rustc version](https://img.shields.io/badge/rustc-1.69+-yellow.svg)
![ISC licensed](https://img.shields.io/badge/license-ISC-blue.svg)

> Rust crate providing cross-platform information about the notebook batteries.

## Table of contents

- [Overview](#overview)
- [Supported platforms](#supported-platforms)
- [Install](#install)
- [Examples](#examples)
- [FFI bindings](#ffi-bindings)
- [Users](#users)
- [License](#license)
- [Donations](#donations)
- [Contributors](#contributors)
- [Backers](#backers)
- [Sponsors](#sponsors)

## Overview

`battery` provides a cross-platform unified API to a notebook batteries state.

Its main goal is to wrap the OS-specific interfaces, cover all the hacks and legacy cases
and get the batteries information (such as state of charge, energy rate, voltage and temperature)
as a typed values, recalculated as necessary to be returned as a [SI measurement units](https://www.bipm.org/en/measurement-units/).

## Supported platforms

- Linux 2.6.39+
- MacOS 10.10+
- iOS
- Windows 7+
- FreeBSD
- DragonFlyBSD

Do note that iOS implementation uses IOKit bindings, your application
might be automatically rejected by Apple based on that fact. Use it on your own risk.

## Install

As a prerequisite, `battery` crate requires at least Rustc version **1.69** or greater.

Add the following line into a `Cargo.toml`:

```toml
[dependencies]
battery = "0.7.8"
```

## Examples

```rust
fn main() -> Result<(), battery::Error> {
    let manager = battery::Manager::new()?;

    for (idx, maybe_battery) in manager.batteries()?.enumerate() {
        let battery = maybe_battery?;
        println!("Battery #{}:", idx);
        println!("Vendor: {:?}", battery.vendor());
        println!("Model: {:?}", battery.model());
        println!("State: {:?}", battery.state());
        println!("Time to full charge: {:?}", battery.time_to_full());
        println!("");
    }

    Ok(())
}
```

See the `battery/examples/` folder in the [repository](https://github.com/starship/rust-battery/blob/main/battery/examples/simple.rs)
for additional examples.

## Users

This an incomplete list of the `battery` crate users. If you are using it too,
send me a message and I'll add your project here!

### starship

[`starship`](https://github.com/starship/starship) is a Rust port of the minimalistic, powerful,
and extremely customizable prompt Spaceship ZSH.\
It is using the `battery` crate to show the the current battery level and status in a shell prompt.

Here is what [@matchai](https://github.com/matchai) says:

> I really appreciate how easily we were able to get your library up and running!
> Battery APIs were a headache for us in predecessors of this project 😅

And there is [this tweet](https://twitter.com/matchai/status/1135906726392283136) also!
