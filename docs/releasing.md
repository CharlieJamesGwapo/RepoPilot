# Release checklist

This source tree includes release automation; no repository or package has been published by generating it.

1. Establish the public repository, maintainers, private security reporting channel, and community contact. Confirm ownership, MIT licensing, and dependency licenses.
2. Review the README and changelog against actual behavior. Replace “Unreleased” with the release date. Confirm `Cargo.toml` version and `Cargo.lock` are committed.
3. Run formatting, strict Clippy, and all tests. Exercise help, core commands without configuration, local-only rejection, mock provider failures, chat navigation, and permission prompts.
4. Run CI on Linux, macOS, and Windows. The workflow files are templates until they have succeeded on the actual hosted repository. Review third-party action revisions and pin them to reviewed immutable commits for your supply-chain policy before launch.
5. Tag the reviewed commit `v0.1.0` and push the tag only when ready to publish. **A matching `v*` tag triggers public GitHub Release creation automatically.** Protect tag creation and restrict workflow modification.
6. Release automation checks the tag against package metadata, tests and builds each native platform, runs binary and JSON smoke checks, packages README/license/changelog, extracts and runs the packaged binary, and verifies SHA-256 checksums before publication.
7. Download each published archive, check its `.sha256`, and test on its target OS. Document tested minimum OS versions. These archives are unsigned; do not claim signing, notarization, reproducibility, universal Linux ABI compatibility, or package-manager availability.
8. Publish registry packages or package-manager formulas only after separate review and explicit maintainer action. Do not invent download URLs or installation commands before hosting exists.

Targets: x86-64 Linux GNU, x86-64 Windows MSVC, Intel macOS, and ARM64 macOS. Linux builds use the hosted runner's glibc baseline; older distributions may require building from source. Runner image availability and GitHub permission policies must be verified when the workflow first runs.
