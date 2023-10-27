# justfile is used for cross-compiling (to Raspberry Pi) purposes only

# Build variables
BUILD_NAME := "dust.bin"
TARGET_NAME := "aarch64-unknown-linux-gnu"
TARGET_DIR := "target-" + TARGET_NAME

# List commands
default:
    @just --list

# Debug build
dbuild:
    cross build --target-dir {{TARGET_DIR}} --target {{TARGET_NAME}} -vv

# Release build
rbuild:
    cross build --release --target-dir {{TARGET_DIR}} --target {{TARGET_NAME}} -vv

# Run debug target
drun: dbuild
    cross run --target-dir {{TARGET_DIR}} --target {{TARGET_NAME}} -vv

# Run release target
rrun: rbuild
    cross run --release --target-dir {{TARGET_DIR}} --target {{TARGET_NAME}} -vv

# Create binary
bin: clean rbuild
    @# cross objcopy --release -vv -- -O binary bin/dust.bin --> FIX ME !!
    cp {{TARGET_DIR}}/{{TARGET_NAME}}/release/dust ./bin/{{BUILD_NAME}}

# Print binary size
size:
    @ls -sh ./bin/dust.bin

# Clean target
clean:
    rm -rf ./bin/*
    rm -rf ./target/
    rm -rf ./target-aarch64-unknown-linux-gnu/
    rm -rf ./target-armv7-unknown-linux-gnueabihf/
    rm -rf /tmp/rustimport/

# Git
git:
    git status
    git diff

# Create rust shared libraries for python
py: clean
    python -m rustimport build --release drishti/
    rm -rf /tmp/rustimport/
    python -m rustimport build --release dust/
    rm -rf /tmp/rustimport/
    python -m rustimport build --release vahana/
    rm -rf /tmp/rustimport/
    cp *.so ./python/
