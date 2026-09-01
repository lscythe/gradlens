#!/bin/sh
set -eu

repo=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)

if ! command -v cargo >/dev/null 2>&1; then
  echo "error: cargo is required; install Rust from https://rustup.rs" >&2
  exit 1
fi

if [ -n "${INSTALL_DIR:-}" ]; then
  install_dir=$INSTALL_DIR
elif [ -n "${PREFIX:-}" ]; then
  install_dir=$PREFIX/bin
else
  install_dir=${CARGO_HOME:-$HOME/.cargo}/bin
fi

cargo build --release --locked --manifest-path "$repo/Cargo.toml"
mkdir -p "$install_dir"
source_binary=$repo/target/release/gradlens
temporary=$install_dir/.gradlens.tmp.$$
trap 'rm -f "$temporary"' EXIT HUP INT TERM
cp "$source_binary" "$temporary"
chmod 755 "$temporary"
mv -f "$temporary" "$install_dir/gradlens"
trap - EXIT HUP INT TERM

echo "Installed gradlens to $install_dir/gradlens"
case ":${PATH:-}:" in
  *":$install_dir:"*) ;;
  *) echo "Add $install_dir to PATH to run gradlens from any directory." ;;
esac
