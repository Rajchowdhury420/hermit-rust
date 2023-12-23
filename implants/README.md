# Hermit Implants

## Build

### Target: Windows

1. Download a toolchain for Windows

```sh
rustup target add x86_64-pc-windows-gnu
```

2. Compile

```sh
cargo build --target x86_64-pc-windows-gnu --release
```

After compiling, the executable is stored in `target/x86_64-pc-windows-gnu/release/implants.exe `.