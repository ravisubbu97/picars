# List commands
default:
  just --list

# Debug build
dbuild:
    cross build

# Release build
rbuild:
    cross build --release

# Run debug target
drun: dbuild
    cross run

# Run release target
rrun: rbuild
    cross run --release

# Copy binary
bin:
    cp target/armv7-unknown-linux-gnueabihf/release/cross-hello bin/

# Print binary size
size:
    cargo size -q --release

# Clean target
clean:
    rm -rf ./target
    rm -rf ./bin
