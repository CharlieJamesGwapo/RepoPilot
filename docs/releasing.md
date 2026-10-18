# Releasing RepoPilot

Repository: [CharlieJamesGwapo/RepoPilot](https://github.com/CharlieJamesGwapo/RepoPilot). Release: [v0.1.0](https://github.com/CharlieJamesGwapo/RepoPilot/releases/tag/v0.1.0).

1. Review the version, changelog, README, installer defaults, and documentation against actual behavior. Commit `Cargo.lock`; confirm dependency licenses and MIT licensing. Establish private security reporting and a community contact.
2. Run formatting, strict Clippy, and all tests. Exercise core diagnostics offline, permission failures, local-only restrictions, saved-local chat, real-shell behavior, and installer failure paths with fixtures.
3. Run hosted CI on Linux, macOS, and Windows. Treat workflow execution as verified only after its actual jobs succeed; a checked-in workflow is not evidence of a passing run.
4. Create the reviewed version tag and push it only when ready to publish. **A matching `v*` tag triggers GitHub Release creation automatically.** Protect tag creation and restrict workflow changes.
5. The release workflow validates the package/tag match, builds the four target archives, smoke-tests packaged executables, and verifies checksums. Distribute `install.sh` and `install.ps1` as release assets alongside archives and `.sha256` files.
6. Download the actual published assets, verify each checksum, and test installation and binary execution on its supported target. Retain job and test evidence before claiming a platform has passed. Check installer asset URLs, version output, PATH guidance, and upgrade behavior.
7. Review release notes and installation documentation. Binaries are unsigned; macOS builds are not notarized. Do not imply signing, universal compatibility, reproducibility, Homebrew core distribution or crates.io publication.

Targets: `aarch64-apple-darwin`, `x86_64-apple-darwin`, `x86_64-unknown-linux-gnu`, and `x86_64-pc-windows-msvc`. Linux builds use Ubuntu 22.04. Unsupported architectures or older runtime libraries may require building from source.

Installers default to version `v0.1.0`. The shell installer supports `REPOPILOT_VERSION` and `REPOPILOT_INSTALL_DIR`; PowerShell supports `-Version` and `-InstallDir`. Both verify downloads before extraction/version validation and atomic replacement, and print PATH instructions without editing user configuration. Update installer defaults deliberately for the next release.

## Homebrew tap maintenance

The separate public [homebrew-tap repository](https://github.com/CharlieJamesGwapo/homebrew-tap) provides `brew install charliejamesgwapo/tap/repopilot`. Keep it distinct from Homebrew core. The formula selects the immutable published macOS v0.1.0 archives by CPU architecture and verifies their published SHA-256 digests; installation needs no Rust toolchain. It generates bash/zsh/fish completions and installs project documentation in Homebrew's documentation directory.

Before merging tap changes, validate installation, `--version`, offline commands, completions, documentation, and uninstall on both Intel and Apple Silicon macOS. Retain actual CI job results before reporting either platform passed. Test the absolute Homebrew binary as well as PATH lookup because an existing local installation may shadow it. Do not delete a user's older binary or install AI tools, accounts, or models.

Homebrew support is an installation/documentation addition, not a new RepoPilot binary release. Preserve the v0.1.0 tag and release assets. Link users to the installation guide on `main` for the latest instructions. A future binary version requires new immutable assets and an intentional formula/version/checksum update.
