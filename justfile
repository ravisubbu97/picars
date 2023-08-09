# List commands
default:
  @just --list

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

# Create binary
bin:
    cargo objcopy --release -q -- -O binary bin/image.bin

# Print binary size
size:
    @ls -sh ./bin/image.bin

# Clean target
clean:
    @cargo clean -vv
    rm -rf ./bin/*
