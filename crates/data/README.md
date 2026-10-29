# Nyanko
[![Crates.io](https://img.shields.io/crates/v/nyanko.svg)](https://crates.io/crates/nyanko)

Pure stateless library for handling game quirks, animations, and data from *The Battle Cats*.

Designed as a standalone API, `nyanko` abstracts away the underlying engine complexities, providing a clean front-facing module for developers and modders to input and process game files.

## Features
- **Unit Structs:** Aggregate structs for both Cat Units and Enemy Units that can be serialized to `.json`, each with an `assemble` constructor that performs the cross-file lookups the engine's layout requires.
- **Data Structs:** Re-exports containing pure-data structs for raw game files.
- **Animation Solving:** Does all the complex animation engine math to give an ambiguous `FrameData` struct for any canvas implementation to consume.
- **Pack Cryptology:** Hands back unencrypted pack bytes when encrypted pack bytes and keys are provided.
- **BCU Cryptology:** Hands back unencrypted bcuzip bytes when an encrypted bcuzip and a key are provided.

## Installation
Type the following command in your terminal next to your `Cargo.toml`:

```bash
cargo add nyanko
```
