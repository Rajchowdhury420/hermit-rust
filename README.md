# Hermit C2

> [!CAUTION]
> I created a Golang version of [Hermit](https://github.com/hideckies/hermit) because I didn't feel the need to write the entire framework in Rust, although I might be going to write the Windows payload in 'windows-rs'.

<br />

A post-exploitation, command and control framework written in Rust.  
I'm developing it to learn how the C2 framework works and learn Rust programming language.

<br />

![diagram](assets/diagram.png)

<br />

## FEATURES

This is still in the early stages of development and still has minimal basic functionality.

- C2 server
- C2 client (CLI)
- Communication between the C2 server and the C2 client over gRPC
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

Please refer to [the documentation](https://hermit.hdks.org/) for more details.

### C2 Server

```sh
$ hermit server


        ┓┏┏┓┳┓┳┳┓┳┏┳┓
        ┣┫┣ ┣┫┃┃┃┃ ┃
        ┛┗┗┛┛┗┛ ┗┻ ┻
          C2 SERVER
      +++++++++++++++++
      DEVELOPED BY HDKS

[2024-01-10T13:59:41Z INFO  hermit::server::server]  Start gRPC server on http://::1:9999
```

### C2 Client

```sh
$ hermit client -P 9999


        ┓┏┏┓┳┓┳┳┓┳┏┳┓
        ┣┫┣ ┣┫┃┃┃┃ ┃
        ┛┗┗┛┛┗┛ ┗┻ ┻
          C2 CLIENT
      +++++++++++++++++
      DEVELOPED BY HDKS

[+] Connected the C2 server (http://[::1]:9999) successfully.

Hermit $
```

To print the usage:

```sh
Hermit $ help
```

<br />

## INSTALLATION

Clone the repository first.

```sh
git clone https://github.com/hideckies/hermit.git
cd hermit
```

### Install Dependencies

**Automatically (recommended):**  

If you use Hermit on Linux system, it's recommended to use `install.sh` to make the installation process quickly and easily.

```sh
chmod +x install.sh
./install.sh
```

**Manually:**  

If you use Hermit on Windows system or would like to install manually, follow the [Installation](https://hermit.hdks.org/docs/installation/) page.

```sh
git clone https://github.com/hideckies/hermit-rust.git
cd hermit
cargo build --release
./target/release/hermit --version
sudo cp ./target/release/hermit /usr/local/bin
```
