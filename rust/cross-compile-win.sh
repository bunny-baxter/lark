# Setup instructions:
# > brew install mingw-w64
# > rustup target add x86_64-pc-windows-gnu
# Then add the following to ~/.cargo/config.toml:
# [target.x86_64-pc-windows-gnu]                                                                
# linker = "x86_64-w64-mingw32-gcc"

cargo build --release --target x86_64-pc-windows-gnu && echo "Successfully compiled to target/x86_64-pc-windows-gnu/release/rust.exe"
