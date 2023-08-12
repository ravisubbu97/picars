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
bin: rbuild
    cargo objcopy --release -vv -- -O binary bin/dust.bin

# Print binary size
size:
    @ls -sh ./bin/dust.bin

# Clean target
clean:
    @cargo clean -vv
    rm -rf ./bin/*
