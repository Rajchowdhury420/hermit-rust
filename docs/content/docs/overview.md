+++
title = "Overview"
date = 2024-01-02
[extra]
toc = true
+++

Hermit is a command and control framework written in Rust.  
I'm developing it to learn how the C2 framework works and learn Rust programming.

![diagram](/diagram.png)

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
- Implant Obfuscation
- And other techniques.

<br />

## STATUS

This project is currently under development.  
It does not have features enough for attack simulations yet, and only the basic feature to communicate with target computer.  
However, I'm working to implement those (and new) features.

<br />

## :warning: WARNING

This project is only be used for educational and learning purposes and for experimentation in your own environment.  
It's prohibited to use on systems not under your control.

