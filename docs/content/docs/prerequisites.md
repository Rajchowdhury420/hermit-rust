+++
title = "Prerequisites"
date = 2024-01-02
[extra]
toc=true
+++

## Install Rust

**Hermit C2** is written in **Rust** so we need **cargo** to build the project for both the C2 server and the C2 client.  
If the **Rust** has not been installed on your system, install it first.

<br />

## Install Proto Buffers Compiler

It's required for serving gRPC with [Tonic](https://github.com/hyperium/tonic) for the C2 server.  

**Debian/Ubuntu:**

```sh
sudo apt install protobuf-compiler libprotobuf-dev
```

**Alpine Linux:**

```sh
sudo apk add protoc protobuf-dev
```

**Windows:**

1. Download from the [release](https://github.com/protocolbuffers/protobuf/releases/latest) page.  
2. Extract the file bin\protoc.exe and put it somewhere in the PATH.
3. Check if the command is available with executing `protoc --version`.


<br />

## For C2 Server Only

C2 server can work on **Linux** and **Windows** (**macOS** is not tested).  
However, it's recommended to use it on **Linux** for stable operations.

### Packages

**Debian/Ubuntu:**

Install the required packages.  

- `mingw-w64`: required for cross-compiling implants for Windows agent.
- `libxcb-xfixes0-dev`: required for compiling implants with the screenshot feature for Linux agent.

```sh
sudo apt install -y git build-essential mingw-w64 libxcb-xfixes0-dev
```

<br />

### Rustup Target for Cross-Compilation

For cross-compilation implants, we need to add the following targets for each agent.

- **Linux target:**

```sh
rustup target add x86_64-unknown-linux-gnu
rustup target add i686-unknown-linux-gnu
```

- **Windows target:**

```sh
rustup target add x86_64-pc-windows-gnu
rustup target add i686-pc-windows-gnu
```

<br />

## For C2 Client Only

Similar to the C2 server, the C2 client also works on **Linux** and **Windows** but recommended to use it on **Linux** for stable operations.  

Fortunately, thera are no prerequisites for running the C2 client except **Rust** as far as I know.