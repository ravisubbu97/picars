# Build variables
BUILD_NAME := "dust.bin"
TARGET_NAME := "armv7-unknown-linux-gnueabihf"
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

# Git
git:
    git status
    git diff
