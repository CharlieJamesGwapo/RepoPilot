#!/usr/bin/env python3
"""Reproducible warm-cache CLI measurements. No repository or network changes."""
import argparse
import hashlib
import json
import pathlib
import platform
import re
import shutil
import statistics
import subprocess
import tempfile
import time


def measure(command, cwd, runs):
    subprocess.run(command, cwd=cwd, check=True, capture_output=True)
    samples = []
    for _ in range(runs):
        start = time.perf_counter_ns()
        subprocess.run(command, cwd=cwd, check=True, capture_output=True)
        samples.append((time.perf_counter_ns() - start) / 1_000_000)
    memory = None
    if platform.system() == "Darwin":
        result = subprocess.run(["/usr/bin/time", "-l", *command], cwd=cwd, check=True, capture_output=True, text=True)
        match = re.search(r"(\d+)\s+maximum resident set size", result.stderr)
        if match:
            memory = int(match.group(1))
    return {"runs": runs, "median_ms": round(statistics.median(samples), 3), "min_ms": round(min(samples), 3), "max_ms": round(max(samples), 3), "peak_rss_bytes_single_run": memory}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", default="target/release/repopilot")
    parser.add_argument("--runs", type=int, default=20)
    parser.add_argument("--files", type=int, default=10_000)
    args = parser.parse_args()
    if not 1 <= args.runs <= 1000 or not 1 <= args.files <= 100_000:
        parser.error("runs must be 1..1000 and files 1..100000")
    root = pathlib.Path(__file__).resolve().parents[1]
    binary = str(pathlib.Path(args.binary).resolve())
    results = {"platform": platform.platform(), "machine": platform.machine(), "version": subprocess.check_output([binary, "--version"], text=True).strip(), "binary_sha256": hashlib.sha256(pathlib.Path(binary).read_bytes()).hexdigest(), "method": "One warmup; sequential subprocess wall-clock samples; warm OS cache; output captured; process startup included; RSS from one separate macOS time -l invocation when available", "measurements": {}}
    results["measurements"]["version"] = measure([binary, "--version"], root, args.runs)
    results["measurements"]["self_inventory"] = measure([binary, "scan", ".", "--json"], root, args.runs)
    if shutil.which("cargo"):
        results["cargo_version"] = subprocess.check_output(["cargo", "--version"], text=True).strip()
        results["measurements"]["cargo_metadata_no_deps_offline"] = measure(["cargo", "metadata", "--no-deps", "--offline", "--format-version", "1"], root, args.runs)
    with tempfile.TemporaryDirectory(prefix="repopilot-bench-") as directory:
        project = pathlib.Path(directory)
        (project / "package.json").write_text(json.dumps({"dependencies": {"react": "19", "next": "15", "typescript": "5"}}))
        (project / ".gitignore").write_text("ignored/\n")
        source = project / "src"
        source.mkdir()
        for index in range(args.files):
            (source / f"file{index}.ts").write_text("export {};\n")
        for dirname in ["ignored", "node_modules", "target"]:
            excluded = project / dirname
            excluded.mkdir()
            for index in range(200):
                nested = excluded / str(index)
                nested.mkdir()
                (nested / "package.json").write_text("malformed")
        report = json.loads(subprocess.check_output([binary, "scan", directory, "--json"], text=True))
        assert report["project"]["frameworks"] == ["Next.js", "React"]
        assert report["project"]["manifests"] == ["package.json"]
        assert report["project"]["files_scanned"] == args.files + 1
        results["fixture"] = {"source_files": args.files, "excluded_manifests": 600, "accuracy_assertions": "passed", "incomplete": report["project"]["truncated"]}
        results["measurements"]["synthetic_inventory"] = measure([binary, "scan", directory, "--json"], root, args.runs)
    print(json.dumps(results, indent=2))


if __name__ == "__main__":
    main()
