# Hermit C2

Command & Control, Post-Exploitation Framework written in Rust.  
I developed this project for learning Rust and how C2 framework works.

<br />

## WARNING

This project is only be used for educational and learning purposes and for experimentation in your own environment.  
It's prohibited to use on systems not under your control.

<br />

## Prerequisites

### Linux

Run the following command in the system which is going to run C2 server.

```sh
sudo apt install -y git mingw-w64
```

<br />

## USAGE

### C2 Server

```sh
hermit server
```

### C2 Client

```sh
hermit client

# Print the usage.
hermit > help
```

<br />

## Installation

```sh
git clone https://github.com/hideckies/hermit.git
cd hermit
cargo install
cargo build
```