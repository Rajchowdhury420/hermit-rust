#!/bin/bash

printf '\033[36m%s\033[m\n' 'Hermit Installation'

# Detect operating system.
# Reference: https://unix.stackexchange.com/a/6348
detect_os() {
    if [ -f /etc/os-release ]; then
        . /etc/os-release
        DETECTED_OS=$NAME
    elif type lsb_release >/dev/null 2>&1; then
        DETECTED_OS=$(lsb_release -si)
    elif [ -f /etc/lsb-release ]; then
        . /etc/lsb-release
        DETECTED_OS=$DISTRIB_ID
    elif [ -f /etc/debian_version ]; then
        DETECTED_OS=Debian
    else
        DETECTED_OS=$(uname -s)
    fi
}

# Update packages for Debian/Ubuntu.
update_apt() {
    echo "Update packages."
    yes | sudo apt update
}

# Install packages for Debian/Ubuntu.
install_pkg_for_debian_ubuntu() {
    echo "Install packages."
    yes | sudo apt install git build-essential mingw-w64 libxcb-xfixes0-dev protobuf-compiler libprotobuf-dev
}

# Update packages for Alpine Linux.
update_apk() {
    echo "Update packages."
    yes | sudo apk update
}

# Install packages for Alpine Linux.
install_pkg_for_alpine_linux() {
    echo "Install packages."
    yes | sudo apk add protoc protobuf-dev
}

# Display the instruction for Windows.
instruct_windows() {
    echo "On Windows, install Proto Buffers Compiler with the following steps:"
    echo "1.Download from the release page."
    echo "Extract the file bin\protoc.exe and put it somewhere in the PATH."
    echo "Check if the command is available with executing 'protoc --version'."
}

# Check if 'cargo' command exists.
cargo_exists() {
    echo "Check if the 'cargo' exists on the system."

    if command -v cargo >/dev/null 2>&1; then
        printf '\033[36m%s\033[m %s\n' '[+]' "'cargo' exists."
        return 0
    else
        printf '\033[31m%s\033[m %s\n' '[x]' "'cargo' does not exists."
        return 1
    fi
}

# Add rustup toolchain for cross compiling implants.
add_rustup_target() {
    echo "Add rustup target for cross-compiling implants."

    # Linux target
    rustup target add x86_64-unknown-linux-gnu
    rustup target add i686-unknown-linux-gnu
    # Windows target
    rustup target add x86_64-pc-windows-gnu
    rustup target add i686-pc-windows-gnu
}

# Build Hermit
build_hermit() {
    echo "Build Hermit."
    cargo build --release
}

detect_os
echo "Your operating system is $DETECTED_OS."

if [[ "$DETECTED_OS" == "Debian" ]] || [[ "$DETECTED_OS" == "Ubuntu" ]]; then
    update_apt
    install_pkg_for_debian_ubuntu
elif [[ "$DETECTED_OS" == "Alpine Linux" ]]; then
    update_apk
    install_pkg_for_alpine_linux
elif [[ "$DETECTED_OS" == "Windows"* ]]; then
    instruct_windows
fi

if cargo_exists; then
    add_rustup_target
    build_hermit
    echo -e "\nThe installation finished."
    printf 'Run \033[36m%s\033[m for the usage.\n' "'./target/release/hermit --help'"
else
    echo ""
    printf '\033[31m%s\033[m %s\n' '[!]' 'Please install Rust first.'
    printf '\033[31m%s\033[m %s\n' '[!]' 'Link: https://www.rust-lang.org/tools/install'
fi
