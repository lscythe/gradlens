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
source_binary=$repo/target/release/gradle-checker
temporary=$install_dir/.gradle-checker.tmp.$$
trap 'rm -f "$temporary"' EXIT HUP INT TERM
cp "$source_binary" "$temporary"
chmod 755 "$temporary"
mv -f "$temporary" "$install_dir/gradle-checker"
trap - EXIT HUP INT TERM

echo "Installed gradle-checker to $install_dir/gradle-checker"
case ":${PATH:-}:" in
  *":$install_dir:"*) ;;
  *) echo "Add $install_dir to PATH to run gradle-checker from any directory." ;;
esac
