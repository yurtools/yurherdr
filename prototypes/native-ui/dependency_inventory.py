#!/usr/bin/env python3
"""Resolve target graphs from this lock; save raw metadata only to ignored evidence.

Requires Python 3.10+ and Cargo 1.100.0. No package installation or execution of
the prototype. License labels are an inventory, never redistribution clearance.
"""
import argparse
import csv
import hashlib
import json
from pathlib import Path
import re
import subprocess

TARGETS = ["x86_64-unknown-linux-gnu", "aarch64-apple-darwin", "x86_64-pc-windows-msvc"]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--evidence", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--offline", action="store_true")
    args = parser.parse_args()
    crate = Path(__file__).resolve().parent
    args.evidence.mkdir(parents=True, exist_ok=True)
    checksums = {}
    for block in (crate / "Cargo.lock").read_text().split("[[package]]")[1:]:
        fields = dict(re.findall(r'^([a-z]+) = "([^"]+)"$', block, re.M))
        checksums[(fields["name"], fields["version"])] = fields.get("checksum", "git/path")
    inventory = {}
    for target in TARGETS:
        command = ["cargo", "metadata", "--manifest-path", str(crate / "Cargo.toml"),
                   "--locked", "--features", "native", "--format-version", "1",
                   "--filter-platform", target]
        if args.offline:
            command.append("--offline")
        result = subprocess.run(command, capture_output=True, check=False)
        (args.evidence / (target + ".stderr.txt")).write_bytes(result.stderr)
        print(target, "cargo metadata exit", result.returncode, flush=True)
        result.check_returncode()
        (args.evidence / (target + ".json")).write_bytes(result.stdout)
        data = json.loads(result.stdout)
        # Metadata includes optional edges not active in the compilation graph.
        # Cargo tree owns target/host and feature selection; do not approximate
        # Cargo's resolver by walking every metadata node.
        tree_command = ["cargo", "tree", "--manifest-path", str(crate / "Cargo.toml"),
                        "--locked", "--features", "native", "--target", target,
                        "-e", "normal,build", "--prefix", "none", "--format", "{p}|{f}"]
        if args.offline:
            tree_command.append("--offline")
        tree = subprocess.run(tree_command, capture_output=True, check=False)
        (args.evidence / (target + ".tree.txt")).write_bytes(tree.stdout)
        (args.evidence / (target + ".tree.stderr.txt")).write_bytes(tree.stderr)
        print(target, "cargo tree exit", tree.returncode, flush=True)
        tree.check_returncode()
        active = {}
        for line in tree.stdout.decode().splitlines():
            match = re.match(r"([^ ]+) v([^ ]+)(?: .*)?\|(.*)", line)
            if match:
                name, version, features = match.groups()
                active.setdefault((name, version), set()).update(features.removesuffix(" (*)").split(","))
        assert active, "could not parse Cargo's package/feature output"
        seen = set()
        for package in data["packages"]:
            pair = (package["name"], package["version"])
            if pair not in active or package["name"] == "yurherdr-native-ui-probe":
                continue
            assert pair not in seen, "ambiguous name/version across sources; inspect manually"
            seen.add(pair)
            key = package["id"]
            if key not in inventory:
                directory = Path(package["manifest_path"]).parent
                terms = []
                for path in sorted(directory.iterdir()):
                    if path.is_file() and path.name.upper().startswith(("LICENSE", "NOTICE", "COPYING")):
                        terms.append(path.name + ":" + hashlib.sha256(path.read_bytes()).hexdigest())
                inventory[key] = dict(package=package["name"], version=package["version"],
                    targets=[], license=package["license"] or "UNRESOLVED: inspect license_file",
                    source=package["source"] or "path", checksum=checksums[(package["name"], package["version"])],
                    root_terms=";".join(terms), features=set())
            inventory[key]["targets"].append(target)
            inventory[key]["features"].update(active[pair] - {""})
    args.output.parent.mkdir(parents=True, exist_ok=True)
    with args.output.open("w", newline="") as output:
        writer = csv.DictWriter(output, lineterminator="\n", fieldnames=["package", "version", "targets", "license", "source", "checksum", "root_terms", "features"])
        writer.writeheader()
        for row in sorted(inventory.values(), key=lambda row: (row["package"], row["version"])):
            row["targets"] = ";".join(row["targets"])
            row["features"] = ";".join(sorted(row["features"]))
            writer.writerow(row)
    forbidden = {"gpui-fast", "gpui-shell", "gpui-webview", "lb-wry", "quickjs-jit"}
    assert not forbidden.intersection(row["package"] for row in inventory.values())
    print(len(inventory), "reachable dependency/version entries; no Fast/shell/webview/QuickJS")


if __name__ == "__main__":
    main()
