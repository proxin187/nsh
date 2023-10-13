
# setup
mkdir ~/.config/nsh
touch ~/.config/nsh/conf.nsh
touch ~/.config/nsh/history.txt

# build
cargo build --release
mv target/release/nsh /usr/bin/nsh


