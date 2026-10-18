"""Download public release assets, verify SHA-256, and test real user-local installation."""
import argparse
import hashlib
import http.server
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import threading
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



def verify_models_and_tokens(binary, directory, env):
    models = json.loads(subprocess.check_output([binary, "models", "list", "--json"], env=env, text=True))
    assert {m["id"] for m in models} == {"light", "balanced", "workstation"}
    config = directory / "explicit local model.toml"
    setup = [binary, "models", "setup", "light", "--output", str(config), "--json"]
    result = json.loads(subprocess.check_output(setup, env=env, text=True))
    assert not result["network_accessed"] and not result["downloaded"]
    visible = json.loads(subprocess.check_output([binary, "--config", str(config), "config", "--json"], env=env, text=True))
    assert visible["providers"]["local-coder"]["model"] == "qwen2.5-coder:1.5b"
    assert visible["providers"]["local-coder"]["endpoint"] == "http://127.0.0.1:11434/v1"
    original = config.read_bytes()
    refused = subprocess.run(setup, env=env, capture_output=True, text=True)
    assert refused.returncode != 0 and config.read_bytes() == original
    received = []

    class Handler(http.server.BaseHTTPRequestHandler):
        def log_message(self, *args):
            pass

        def do_POST(self):
            body = json.loads(self.rfile.read(int(self.headers["Content-Length"])))
            received.append(body)
            payload = json.dumps({"choices": [{"message": {"content": "OK"}, "finish_reason": "stop"}], "usage": {"prompt_tokens": 4, "completion_tokens": 3, "total_tokens": 7}}).encode()
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(payload)))
            self.end_headers()
            self.wfile.write(payload)

    server = http.server.HTTPServer(("127.0.0.1", 0), Handler)
    server.timeout = 10
    port = server.server_address[1]
    config.write_text(config.read_text().replace("127.0.0.1:11434", "127.0.0.1:" + str(port)))
    thread = threading.Thread(target=server.handle_request, daemon=True)
    thread.start()
    try:
        answer = json.loads(subprocess.check_output([binary, "--config", str(config), "ask", "release protocol fixture", "--local", "--max-output-tokens", "3", "--json"], env=env, text=True, timeout=20))
        assert answer["answer"] == "OK" and answer["output_limit"] == 3
        assert answer["usage"]["completion_tokens"] == 3 and answer["finish_reason"] == "stop"
    finally:
        thread.join(timeout=12)
        server.server_close()
    assert not thread.is_alive() and len(received) == 1
    assert received[0]["max_tokens"] == 3
    assert received[0]["messages"] == [{"role": "user", "content": "release protocol fixture"}]
    return ["offline local-model catalog and explicit loopback config setup; overwrite refused", "response-budget request and provider usage metadata verified against a local protocol fixture"]


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
        checks.extend(verify_models_and_tokens(binary, directory, env))
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
