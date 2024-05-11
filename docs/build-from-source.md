# How to build Workbench from source

Start by installing these dependencies:

- [Rust](https://rustup.rs/)

Run these commands:

```shell
# Clone the repository
git clone https://github.com/sophie-katz/workbench.git

# Change to the repository directory
cd workbench

# Build the project
cargo build --locked --release

# Install the binary
mkdir -p ~/.workbench/bin
cp target/release/wb ~/.workbench/bin

# Update shell startup script (replace ~/.bashrc with your shell's startup script if necessary)
echo >> ~/.bashrc
echo 'export WORKBENCH_DIR="$HOME/.workbench"' >> ~/.bashrc
echo 'export PATH="$WORKBENCH_DIR:$PATH"' >> ~/.bashrc
```
