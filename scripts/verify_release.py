"""Download public release assets, verify SHA-256, and test real user-local installation."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import time
import urllib.request


def download(url, destination):
    for attempt in range(4):
        try:
            with urllib.request.urlopen(urllib.request.Request(url, headers={"User-Agent": "RepoPilot-release-verification"}), timeout=90) as response:
                destination.write_bytes(response.read())
            return
        except Exception:
            if attempt == 3:
                raise
            time.sleep(3 * (attempt + 1))


def verified_download(base, name, directory):
    path = directory / name
    download(base + "/" + name, path)
    checksum = directory / (name + ".sha256")
    download(base + "/" + name + ".sha256", checksum)
    expected = checksum.read_text().split()[0]
    actual = hashlib.sha256(path.read_bytes()).hexdigest()
    if len(expected) != 64 or actual != expected:
        raise AssertionError("SHA-256 mismatch: " + name)
    return path, actual


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--version", required=True)
    parser.add_argument("--target", required=True)
    parser.add_argument("--report", required=True)
    args = parser.parse_args()
    base = "https://github.com/CharlieJamesGwapo/RepoPilot/releases/download/" + args.version
    windows = os.name == "nt"
    checks = []
    with tempfile.TemporaryDirectory(prefix="repopilot-release-test-") as location:
        directory = Path(location)
        asset = "repopilot-" + args.version + "-" + args.target + (".zip" if windows else ".tar.gz")
        _, digest = verified_download(base, asset, directory)
        checks.append("published archive SHA-256 verified")
        script_name = "install.ps1" if windows else "install.sh"
        installer, _ = verified_download(base, script_name, directory)
        source_installer = Path(__file__).resolve().parents[1] / script_name
        assert installer.read_bytes() == source_installer.read_bytes(), "Published installer differs from tagged source"
        checks.append("published installer checksum and tagged source match")
        install_dir = directory / "local bin with spaces"
        env = dict(os.environ)
        if windows:
            command = ["powershell.exe", "-NoProfile", "-ExecutionPolicy", "Bypass", "-File", str(installer), "-Version", args.version, "-InstallDir", str(install_dir)]
        else:
            env.update(REPOPILOT_VERSION=args.version, REPOPILOT_INSTALL_DIR=str(install_dir))
            command = ["sh", str(installer)]
        subprocess.run(command, env=env, check=True)
        checks.append("downloaded installer installed into a directory with spaces")
        env["PATH"] = str(install_dir) + os.pathsep + env["PATH"]
        binary = shutil.which("repopilot", path=env["PATH"])
        assert binary and Path(binary).parent == install_dir
        version = subprocess.check_output([binary, "--version"], env=env, text=True).strip()
        assert version == "repopilot " + args.version.removeprefix("v")
        checks.append("installed command resolves through PATH and reports expected version")
        for arguments in [["--help"], ["welcome"], ["guide", "undo", "--json"]]:
            output = subprocess.check_output([binary, *arguments], env=env, text=True)
            if "--json" in arguments:
                assert json.loads(output)["id"] == "undo"
        fixture = directory / "empty project"
        fixture.mkdir()
        report = subprocess.check_output([binary, "scan", str(fixture), "--json"], env=env, text=True)
        assert json.loads(report)["schema_version"] == 1
        checks.append("help, welcome, offline guide and project JSON scan work")
        skills = json.loads(subprocess.check_output([binary, "skills", "list", "--json"], env=env, text=True))
        assert len(skills) == 10
        for target in ["portable", "codex", "claude"]:
            command_args = [binary, "skills", "export", "debugging", "--path", str(fixture), "--target", target, "--json"]
            planned = json.loads(subprocess.check_output([*command_args, "--dry-run"], env=env, text=True))
            assert planned["status"] == "planned" and not Path(planned["destination"]).exists()
            exported = json.loads(subprocess.check_output(command_args, env=env, text=True))
            dest = Path(exported["destination"])
            assert "name: repopilot-debugging" in (dest / "SKILL.md").read_text()
            assert "Jesse Vincent" in (dest / "LICENSE.txt").read_text()
            assert "Superpowers" in (dest / "NOTICE.md").read_text()
            refused = subprocess.run(command_args, env=env, capture_output=True, text=True)
            assert refused.returncode != 0 and "already exists" in refused.stderr
        checks.append("ten built-in skills; all three project export targets, dry-run, attribution and overwrite refusal work")
        env["SHELL"] = "/bin/sh"
        script = "echo REPOPILOT_INSTALL_SHELL_OK\r\nexit /b 0\r\n" if windows else "printf 'REPOPILOT_INSTALL_SHELL_OK\\n'\nexit 0\n"
        result = subprocess.run([binary, "shell", "--plain-prompt"], input=script, capture_output=True, text=True, env=env, check=True)
        assert "REPOPILOT_INSTALL_SHELL_OK" in result.stdout
        checks.append("installed real shell executes normal commands")
        subprocess.run(command, env=env, check=True)
        checks.append("existing installation updates successfully")
    result = {"target": args.target, "version": args.version, "download_sha256": digest, "passed": len(checks), "failed": 0, "checks": checks}
    Path(args.report).write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps(result, indent=2))


if __name__ == "__main__":
    main()
