+++
title = "Installation"
date = 2024-01-02
[extra]
toc = true
+++

## Install Automatically (Recommended)

If you're going to use Hermit on **Linux**, it's recommended to run `install.sh` to make the installation process quickly and easily.

```sh
git clone https://github.com/hideckies/hermit.git
cd hermit
chmod +x install.sh
./install.sh
```

<br />

## Install Manually

If you're going to use Hermit on **Windows** or would like to install manually, follow the steps:

### 1. Install Rust (If not installed)

See [the official guide](https://www.rust-lang.org/tools/install).  
Check the `cargo` is installed after that:

```sh
cargo version
```

### 2. Install Packages

**Debian/Ubuntu:**

```sh
sudo apt install git build-essential mingw-w64 libxcb-xfixes0-dev protobuf-compiler libprotobuf-dev
```

**Alpine Linux:**

```sh
sudo apk add git build-base mingw-w64-gcc protoc protobuf-dev
```

**Windows:**

1. Download Proto Buffers Compiler from the [release](https://github.com/protocolbuffers/protobuf/releases/latest) page.  
2. Extract the file bin\protoc.exe and put it somewhere in the PATH.
3. Check if the command is available with executing `protoc --version`.


### 3. Add Rustup Target

For cross-compiling implants, the following toolchains need to be added.

```sh
rustup target add x86_64-unknown-linux-gnu
rustup target add i686-unknown-linux-gnu
rustup target add x86_64-pc-windows-gnu
rustup target add i686-pc-windows-gnu
```

### 4. Clone Repository

Now it's time to use Hermit.  
Clone the repository and build it.

```sh
git clone https://github.com/hideckies/hermit.git
cd hermit
cargo build --release
./target/release/hermit --version
```
