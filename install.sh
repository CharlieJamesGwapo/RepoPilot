#!/bin/sh
# Install a checksum-verified RepoPilot release into a user-owned directory.
set -eu
version=${REPOPILOT_VERSION:-v0.2.0}
install_dir=${REPOPILOT_INSTALL_DIR:-"$HOME/.local/bin"}
case "$version" in v[0-9]*.[0-9]*.[0-9]*) ;; *) echo 'Expected a version tag such as v0.1.0.' >&2; exit 1 ;; esac
case "$version" in *[!a-zA-Z0-9._+-]*) echo 'Invalid version tag.' >&2; exit 1 ;; esac
case "$(uname -s)/$(uname -m)" in
  Darwin/arm64|Darwin/aarch64) target=aarch64-apple-darwin ;;
  Darwin/x86_64) target=x86_64-apple-darwin ;;
  Linux/x86_64|Linux/amd64) target=x86_64-unknown-linux-gnu ;;
  *) echo 'Supported binaries: Apple Silicon/Intel macOS and Linux x64 (glibc). See docs/installation.md for source builds.' >&2; exit 1 ;;
esac
command -v curl >/dev/null 2>&1 || { echo 'curl is required.' >&2; exit 1; }
command -v tar >/dev/null 2>&1 || { echo 'tar is required.' >&2; exit 1; }
if command -v shasum >/dev/null 2>&1; then hasher=shasum
elif command -v sha256sum >/dev/null 2>&1; then hasher=sha256sum
else echo 'shasum or sha256sum is required to verify the download.' >&2; exit 1
fi
workdir=$(mktemp -d)
install_tmp=''
cleanup() {
  rm -rf -- "$workdir"
  if [ -n "$install_tmp" ]; then rm -f -- "$install_tmp"; fi
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM HUP
name="repopilot-$version-$target"
asset="$name.tar.gz"
base="https://github.com/CharlieJamesGwapo/RepoPilot/releases/download/$version"
printf 'Downloading RepoPilot %s for %s...\n' "$version" "$target"
curl --fail --silent --show-error --location --proto '=https' --tlsv1.2 --connect-timeout 15 --max-time 180 "$base/$asset" -o "$workdir/$asset"
curl --fail --silent --show-error --location --proto '=https' --tlsv1.2 --connect-timeout 15 --max-time 180 "$base/$asset.sha256" -o "$workdir/checksum"
expected=$(awk 'NR == 1 {print $1}' "$workdir/checksum")
case "$expected" in *[!0-9a-fA-F]*|'') echo 'Invalid SHA-256 checksum file.' >&2; exit 1 ;; esac
[ "${#expected}" -eq 64 ] || { echo 'Invalid SHA-256 checksum length.' >&2; exit 1; }
if [ "$hasher" = shasum ]; then actual=$(shasum -a 256 "$workdir/$asset" | awk '{print $1}')
else actual=$(sha256sum "$workdir/$asset" | awk '{print $1}')
fi
[ "$actual" = "$expected" ] || { echo 'SHA-256 mismatch; nothing was installed.' >&2; exit 1; }
# Extract only the executable as a stream, ignoring archive links and other paths.
tar -xOzf "$workdir/$asset" "$name/repopilot" > "$workdir/repopilot"
chmod 755 "$workdir/repopilot"
[ "$("$workdir/repopilot" --version)" = "repopilot ${version#v}" ] || { echo 'Downloaded binary version did not match; nothing was installed.' >&2; exit 1; }
mkdir -p -- "$install_dir"
[ ! -d "$install_dir/repopilot" ] || { echo 'The destination repopilot path is a directory.' >&2; exit 1; }
install_tmp=$(mktemp "$install_dir/.repopilot-install.XXXXXX")
cp -- "$workdir/repopilot" "$install_tmp"
chmod 755 "$install_tmp"
mv -f -- "$install_tmp" "$install_dir/repopilot"
install_tmp=''
printf 'Installed %s\n' "$install_dir/repopilot"
case ":$PATH:" in *":$install_dir:"*) ;; *) printf 'Add this directory to your shell PATH: %s\n' "$install_dir" ;; esac
printf 'Run repopilot to open your shell, or repopilot --help to explore.\n'
