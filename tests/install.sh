#!/bin/sh
set -eu

repo=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT HUP INT TERM
mkdir -p "$tmp/bin" "$tmp/install" "$tmp/target/release"
printf '#!/bin/sh\nexit 0\n' > "$tmp/target/release/gradle-checker"
chmod +x "$tmp/target/release/gradle-checker"
cat > "$tmp/bin/cargo" <<EOF
#!/bin/sh
mkdir -p '$repo/target/release'
cp '$tmp/target/release/gradle-checker' '$repo/target/release/gradle-checker'
EOF
chmod +x "$tmp/bin/cargo"
PATH="$tmp/bin:$PATH" INSTALL_DIR="$tmp/install" "$repo/install.sh"
test -x "$tmp/install/gradle-checker"
