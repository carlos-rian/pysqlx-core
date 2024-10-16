#!/bin/bash

# Detect the Linux distribution
if [ -f /etc/os-release ]; then
    . /etc/os-release
    DISTRO=$ID
else
    echo "Unable to detect the Linux distribution."
    exit 1
fi

# Install packages based on the detected distro
case $DISTRO in
    arch)
        echo "Detected Arch Linux"
        sudo pacman -S pkgconf openssl
        ;;
    debian|ubuntu)
        echo "Detected Debian or Ubuntu"
        sudo apt-get install -y pkg-config libssl-dev
        ;;
    fedora)
        echo "Detected Fedora"
        sudo dnf install -y pkgconf perl-FindBin perl-IPC-Cmd openssl-devel
        ;;
    alpine)
        echo "Detected Alpine Linux"
        sudo apk add pkgconf openssl-dev
        ;;
    opensuse|suse)
        echo "Detected openSUSE"
        sudo zypper install -y libopenssl-devel
        ;;
    centos)
        echo "Detected CentOS"
        sudo yum install -y pkgconf openssl-devel
        ;;
    *)
        echo "Unsupported Linux distribution: $DISTRO"
        exit 1
        ;;
esac

echo "Installation complete!"
