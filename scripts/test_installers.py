"""Offline installer regression checks. Uses disposable download fixtures, never GitHub."""
import hashlib
import io
import os
from pathlib import Path
import platform
import shutil
import subprocess
import tarfile
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]


@unittest.skipIf(os.name == "nt", "POSIX installer tests; Windows installation is verified on its release runner")
class InstallerTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.fixture = self.root / "downloads"
        self.fixture.mkdir()
        self.tools = self.root / "tools"
        self.tools.mkdir()
        curl = self.tools / "curl"
        curl.write_text("#!/usr/bin/env python3\nimport os,pathlib,shutil,sys\na=sys.argv[1:]\nu=next(x for x in a if x.startswith('https://'))\nshutil.copyfile(pathlib.Path(os.environ['RP_FIXTURE_DIR'])/u.rsplit('/',1)[1],a[a.index('-o')+1])\n")
        curl.chmod(0o755)
        self.install = self.root / "bin with spaces"
        self.env = dict(os.environ, PATH=str(self.tools) + os.pathsep + os.environ["PATH"], RP_FIXTURE_DIR=str(self.fixture), REPOPILOT_INSTALL_DIR=str(self.install), REPOPILOT_VERSION="v0.1.0")
        machine = platform.machine().lower()
        target = "aarch64-apple-darwin" if platform.system() == "Darwin" and machine in {"arm64", "aarch64"} else "x86_64-apple-darwin" if platform.system() == "Darwin" else "x86_64-unknown-linux-gnu"
        self.name = "repopilot-v0.1.0-" + target
        self.asset = self.name + ".tar.gz"
        self.package()

    def package(self, version="0.1.0"):
        path = self.fixture / self.asset
        payload = ("#!/bin/sh\nprintf 'repopilot " + version + "\\n'\n").encode()
        with tarfile.open(path, "w:gz") as archive:
            info = tarfile.TarInfo(self.name + "/repopilot")
            info.size, info.mode = len(payload), 0o755
            archive.addfile(info, io.BytesIO(payload))
            other = tarfile.TarInfo("../../REPOPILOT_SHOULD_NOT_EXIST")
            other.size = 1
            archive.addfile(other, io.BytesIO(b"x"))
        (self.fixture / (self.asset + ".sha256")).write_text(hashlib.sha256(path.read_bytes()).hexdigest() + "  " + self.asset + "\n")

    def run_install(self):
        return subprocess.run(["sh", str(ROOT / "install.sh")], env=self.env, capture_output=True, text=True)

    def existing_binary(self):
        self.install.mkdir()
        destination = self.install / "repopilot"
        destination.write_bytes(b"old installation")
        return destination

    def test_install_with_spaces_and_exact_extraction(self):
        result = self.run_install()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("Installed", result.stdout)
        output = subprocess.check_output([str(self.install / "repopilot"), "--version"], text=True)
        self.assertEqual(output, "repopilot 0.1.0\n")
        self.assertFalse((self.root / "REPOPILOT_SHOULD_NOT_EXIST").exists())
        self.assertFalse(list(self.install.glob(".repopilot-install.*")))

    def test_bad_checksum_keeps_existing_install(self):
        destination = self.existing_binary()
        (self.fixture / (self.asset + ".sha256")).write_text("0" * 64)
        result = self.run_install()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("SHA-256 mismatch", result.stderr)
        self.assertEqual(destination.read_bytes(), b"old installation")

    def test_wrong_binary_version_keeps_existing_install(self):
        destination = self.existing_binary()
        self.package("9.9.9")
        result = self.run_install()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("version did not match", result.stderr)
        self.assertEqual(destination.read_bytes(), b"old installation")

    def download_timeout(self, suffix):
        destination = self.existing_binary()
        curl = self.tools / "curl"
        curl.write_text("""#!/usr/bin/env python3
import os,pathlib,shutil,sys
args=sys.argv[1:]
url=next(x for x in args if x.startswith('https://'))
if url.endswith(os.environ['RP_TIMEOUT_SUFFIX']):
    # Only simulate the bounded failure when timeout policy is actually supplied.
    if '--connect-timeout' not in args or '--max-time' not in args:
        print('download would wait without configured timeout', file=sys.stderr)
        sys.exit(99)
    assert 0 < int(args[args.index('--connect-timeout')+1]) <= 30
    assert 0 < int(args[args.index('--max-time')+1]) <= 300
    print('curl: (28) Operation timed out', file=sys.stderr)
    sys.exit(28)
shutil.copyfile(pathlib.Path(os.environ['RP_FIXTURE_DIR'])/url.rsplit('/',1)[1],args[args.index('-o')+1])
""")
        self.env['RP_TIMEOUT_SUFFIX'] = suffix
        result = self.run_install()
        self.assertEqual(result.returncode, 28, result.stderr)
        self.assertIn('timed out', result.stderr)
        self.assertEqual(destination.read_bytes(), b'old installation')
        self.assertFalse(list(self.install.glob('.repopilot-install.*')))

    def test_archive_download_timeout_keeps_existing_install(self):
        self.download_timeout('.tar.gz')

    def test_checksum_download_timeout_keeps_existing_install(self):
        self.download_timeout('.sha256')

    def test_invalid_version_is_rejected_before_install(self):
        self.env["REPOPILOT_VERSION"] = "v0.1.0/../../bad"
        result = self.run_install()
        self.assertNotEqual(result.returncode, 0)
        self.assertFalse(self.install.exists())

    def test_unsupported_platform_is_actionable(self):
        uname = self.tools / "uname"
        uname.write_text("#!/bin/sh\nprintf 'unsupported\\n'\n")
        uname.chmod(0o755)
        result = self.run_install()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("Supported binaries", result.stderr)


if __name__ == "__main__":
    unittest.main()
