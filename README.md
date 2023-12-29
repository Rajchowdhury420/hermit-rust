# Hermit C2

Command & Control, Post-Exploitation Framework written in Rust.  
I developed this project for learning Rust and how C2 framework works.

<br />

## WARNING

This project is only be used for educational and learning purposes and for experimentation in your own environment.  
It's prohibited to use on systems not under your control.

<br />

## Prerequisites

### C2 Server

```sh
# For Linux agent
rustup target add x86_64-unknown-linux-gnu
rustup target add i686-unknown-linux-gnu

# For Windows agent
rustup target add x86_64-pc-windows-gnu
rustup target add i686-pc-windows-gnu
```

- **Linux**

Run the following command in the system which is going to run C2 server.

```sh
sudo apt install -y git build-essential mingw-w64
```

<br />

## USAGE

### C2 Server

```sh
hermit server
```

### C2 Client

```sh
hermit client -H your-c2-server.com -P 80

# Print the usage.
Hermit $ help
```

<br />

## Installation

```sh
git clone https://github.com/hideckies/hermit.git
cd hermit
cargo build --release
# After compiling, the `hermit` binary will be stored at `target/release/` folder.
```