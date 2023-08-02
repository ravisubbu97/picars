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

# Print binary size
size:
    cargo size -q --release

# Clean target
clean:
    rm -rf ./target
