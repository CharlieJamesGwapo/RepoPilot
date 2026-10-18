"""Package and smoke-test a native release executable and documentation."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import tempfile


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--version", required=True)
    parser.add_argument("--target", required=True)
    parser.add_argument("--binary", required=True)
    parser.add_argument("--output", default="dist")
    args = parser.parse_args()
    source = Path(args.binary).resolve()
    expected = "repopilot " + args.version.removeprefix("v")
    assert subprocess.check_output([str(source), "--version"], text=True).strip() == expected
    result = subprocess.check_output([str(source), "scan", ".", "--json"], text=True)
    assert json.loads(result)["schema_version"] == 1
    dist = Path(args.output).resolve()
    dist.mkdir(parents=True, exist_ok=True)
    name = "repopilot-" + args.version + "-" + args.target
    with tempfile.TemporaryDirectory() as directory:
        stage = Path(directory) / name
        stage.mkdir()
        shutil.copy2(source, stage / source.name)
        for filename in ["README.md", "LICENSE", "CHANGELOG.md", "THIRD_PARTY_NOTICES.md", "install.sh", "install.ps1"]:
            shutil.copy2(filename, stage / filename)
        shutil.copytree("docs", stage / "docs")
        shutil.copytree("licenses", stage / "licenses")
        archive = Path(shutil.make_archive(str(dist / name), "zip" if source.name.endswith(".exe") else "gztar", root_dir=stage.parent, base_dir=name))
    digest = hashlib.sha256(archive.read_bytes()).hexdigest()
    (dist / (archive.name + ".sha256")).write_text(digest + "  " + archive.name + "\n")
    with tempfile.TemporaryDirectory() as location:
        shutil.unpack_archive(str(archive), location)
        unpacked = Path(location) / name
        assert (unpacked / "THIRD_PARTY_NOTICES.md").is_file()
        assert "Jesse Vincent" in (unpacked / "licenses/superpowers-MIT.txt").read_text()
        extracted = unpacked / source.name
        extracted.chmod(extracted.stat().st_mode | 0o111)
        assert subprocess.check_output([str(extracted), "--version"], text=True).strip() == expected
        json.loads(subprocess.check_output([str(extracted), "guide", "undo", "--json"], text=True))
    print("Packaged and verified " + archive.name + ": " + digest)


if __name__ == "__main__":
    main()
