# ⚡ ElectroDynamix

A simple zero-player game, made with **Rust** and **[Bevy](https://bevyengine.org/)**.

[![Latest release](https://img.shields.io/github/v/release/SinedKadis/ElectroDynamix)](https://github.com/SinedKadis/ElectroDynamix/releases/latest)
![Rust](https://img.shields.io/badge/Rust-2024_edition-orange?logo=rust)
![Bevy](https://img.shields.io/badge/Bevy-0.20-blue)

<img width="993" height="843" alt="image" src="https://github.com/user-attachments/assets/be4be08c-9e5b-44bb-ba40-457ec6ab03ee" />


## About

ElectroDynamix is a zero-player simulation: you set up the initial state, then watch it evolve on its own.

<!-- TODO: 2–3 sentences about the rules of the simulation -->

## Features

- Real-time simulation rendered with Bevy and `bevy_vector_shapes`
- Copy and paste a selection via the system clipboard
- Adjustable simulation speed
- Parallelized computation (Rayon)
- Save/load support via JSON (Serde)

## Download

Pre-built binaries are available on the [Releases page](https://github.com/SinedKadis/ElectroDynamix/releases/latest).

1. Download the archive for your platform.
2. Unpack it.
3. Run the executable.

## Build from source

**Requirements**

- [Rust](https://www.rust-lang.org/tools/install) 1.85 or newer (the project uses the 2024 edition)
- Platform dependencies for Bevy — see the [Bevy setup guide](https://bevyengine.org/learn/quick-start/getting-started/setup/) (mostly relevant on Linux)

**Steps**

```bash
git clone https://github.com/SinedKadis/ElectroDynamix.git
cd ElectroDynamix
cargo run --release
```

## Controls

<!-- TODO: fill in with the actual bindings -->

| Action | Key / Mouse |
| ------ | ----------- |
| Select area | RMB |
| Copy selection | Crtl C |
| Paste selection | Crtl V |
| Change speed | Move Slider |
| Pause / resume | Space |

## Changelog

### v0.2.0

- Added copy/paste of the selection to the clipboard
- Fixed the speed cap
- Several computing and rendering optimizations

See all versions on the [Releases page](https://github.com/SinedKadis/ElectroDynamix/releases).

## Tech stack

| Crate | Purpose |
| ----- | ------- |
| [`bevy`](https://crates.io/crates/bevy) | Game engine |
| [`bevy_vector_shapes`](https://crates.io/crates/bevy_vector_shapes) | Vector shape rendering |
| [`serde`](https://crates.io/crates/serde), [`serde_json`](https://crates.io/crates/serde_json) | Serialization |
| [`arboard`](https://crates.io/crates/arboard) | Clipboard access |
| [`rayon`](https://crates.io/crates/rayon) | Parallel computation |

## Contributing

Issues and pull requests are welcome.

## License

MIT
