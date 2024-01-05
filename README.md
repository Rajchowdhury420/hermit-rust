# Hermit C2

A post-exploitation, command and control framework written in Rust.  
I'm developing it to learn how the C2 framework works and learn Rust programming language.

![diagram](assets/diagram.png)

<br />

## FEATURES

This is still in the early stages of development and still has minimal basic functionality.

- C2 server
- C2 client (CLI)
- HTTPS listener
- Implant generation
- AES encryption for each message
- Multi listeners, agents, operators
- Database (SQLite) for the settings persistence

<br />

## UNSUPPORTED FEATURES (YET)

- AV/EDR evasion
- Implant obfuscation
- And other techniques.

<br />

## STATUS

This project is currently under development.  
It does not have features enough for attack simulations yet, and only the basic feature to communicate with target computer.  
However, I'm working to implement those (or new) features.

<br />

## :warning: WARNING

This project is only be used for educational and learning purposes and for experimentation in your own environment.  
It's prohibited to use on systems not under your control.

<br />

## USAGE

<!-- Plese refer to [https://hermit.hdks.org/](the docs) for more details. -->
Plese refer to [the documentation](/docs) for more details.

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