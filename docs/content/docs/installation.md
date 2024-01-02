+++
title = "Installation"
date = 2024-01-02
[extra]
toc = true
+++

Before installation, please check [the prerequisites](/docs/prerequisites) at first.

## Build from Source

```sh
git clone https://github.com/hideckies/hermit.git
cd hermit
cargo build --release
```

After building, the `hermit` binary will be stored under `target/release/` directory.  
We can execute it as the following:

```sh
./target/release/hermit --version
```
