#!/bin/sh
set -eu

repo=${GRADLENS_REPOSITORY:-lscythe/gradlens}
version=${GRADLENS_VERSION:-latest}
install_dir=${INSTALL_DIR:-${CARGO_HOME:-$HOME/.cargo}/bin}
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT HUP INT TERM

case $(uname -s) in
  Linux) os=unknown-linux-gnu ;;
  Darwin) os=apple-darwin ;;
  *) echo "error: unsupported operating system: $(uname -s)" >&2; exit 1 ;;
esac
case $(uname -m) in
  x86_64|amd64) arch=x86_64 ;;
  arm64|aarch64) arch=aarch64 ;;
  *) echo "error: unsupported architecture: $(uname -m)" >&2; exit 1 ;;
esac

target=$arch-$os
base=https://github.com/$repo/releases
if [ "$version" = latest ]; then
  base=$base/latest/download
  asset=gradlens-$target.tar.gz
else
  base=$base/download/$version
  asset=gradlens-$version-$target.tar.gz
fi

command -v curl >/dev/null 2>&1 || { echo "error: curl is required" >&2; exit 1; }
curl --proto '=https' --tlsv1.2 -fLsS "$base/$asset" -o "$tmp/$asset"
curl --proto '=https' --tlsv1.2 -fLsS "$base/$asset.sha256" -o "$tmp/$asset.sha256"
(cd "$tmp" && shasum -a 256 -c "$asset.sha256")
tar -xzf "$tmp/$asset" -C "$tmp"
binary=$(find "$tmp" -type f -name gradlens -perm -u+x | head -n 1)
[ -n "$binary" ] || { echo "error: archive did not contain gradlens" >&2; exit 1; }
mkdir -p "$install_dir"
cp "$binary" "$install_dir/.gradlens.tmp.$$"
chmod 755 "$install_dir/.gradlens.tmp.$$"
mv -f "$install_dir/.gradlens.tmp.$$" "$install_dir/gradlens"
echo "Installed gradlens to $install_dir/gradlens"
