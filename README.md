# Hermit C2

A post-exploitation, command and control framework written in Rust.  
I'm developing it to learn how C2 framework works and learn Rust programming language.

<br />

## FEATURES

![diagram](assets/diagram.png)

- C2 server
- C2 client (CLI)
- HTTP listener
- Implant generation
- AES encryption for communication
- Multi listeners, agents, operators

<br />

## STATUS

This project is currently under development.  
It does not have features enough for attack simulations yet, and only the basic feature to communicate with target computer.  
However, I'm working to add those (or new) features.

<br />

## :warning: WARNING

This project is only be used for educational and learning purposes and for experimentation in your own environment.  
It's prohibited to use on systems not under your control.

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
```

<br />

## INSTALLATION

```sh
git clone https://github.com/hideckies/hermit.git
cd hermit
cargo build --release
./target/release/hermit --version
```