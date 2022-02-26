# La Bevy Jam #1

Our participation to [the Bevy Jam #1](https://itch.io/jam/bevy-jam-1).

## Installation

```bash
rustup target install wasm32-unknown-unknown

cargo build --target wasm32-unknown-unknown

cargo install wasm-server-runner cargo-watch
```

```bash
cargo watch -cx 'run'

# You can also debug the 2d heron/rapier collision boxes
cargo watch -cx 'run --features debug-2d'
```
