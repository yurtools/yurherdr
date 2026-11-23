#!/usr/bin/env python3
"""Stage an explicitly verified local build in a fresh guarded run. Never fetches.

The supplied SHA-256 must come from the independently pinned build evidence.
This receipt binds bytes to that evidence; it is not a signed source attestation.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess

CORE_REVISION = "300a455b8cfa91f8f068b49bb9d78b94d3bd4f0e"
GHOSTTY_REVISION = "1e5cdb865807abc990181efb8ebfa9122955cce2"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--probe", required=True, type=Path)
    parser.add_argument("--core", required=True, type=Path)
    parser.add_argument("--run-root", required=True, type=Path)
    parser.add_argument("--nonce", required=True)
    parser.add_argument("--sha256", required=True)
    args = parser.parse_args()
    if "HC_SESSION" in os.environ:
        parser.error("stage outside the inherited live core session")
    if not all(p.is_absolute() for p in [args.probe, args.core, args.run_root]):
        parser.error("all paths must be absolute")
    if len(args.sha256) != 64 or any(c not in "0123456789abcdef" for c in args.sha256):
        parser.error("supply the exact lowercase SHA-256 from the build record")
    subprocess.run([str(args.probe), "--guard-only", str(args.run_root), "--nonce", args.nonce], check=True)
    destination = args.run_root / "core"
    if list(destination.iterdir()):
        parser.error("core directory must be empty; use a fresh run")
    # Keep one source descriptor across hashing/copying; refuse symlinks.
    flags = os.O_RDONLY | getattr(os, "O_NOFOLLOW", 0) | getattr(os, "O_NONBLOCK", 0)
    with os.fdopen(os.open(args.core, flags), "rb") as source:
        import stat
        metadata = os.fstat(source.fileno())
        if not stat.S_ISREG(metadata.st_mode) or metadata.st_size > 512 * 1024 * 1024:
            parser.error("core must be a bounded regular file")
        digest = hashlib.sha256()
        while block := source.read(65536):
            digest.update(block)
        if digest.hexdigest() != args.sha256:
            parser.error("core bytes differ from the build record")
        source.seek(0)
        # Partial provisioning deliberately leaves marked evidence. No unknown
        # files are overwritten/deleted and no core is started by this script.
        with (destination / "herdr-core").open("xb") as binary:
            os.chmod(binary.fileno(), 0o700)
            shutil.copyfileobj(source, binary)
            binary.flush()
            os.fsync(binary.fileno())
    with (destination / "build.json").open("x") as receipt:
        os.chmod(receipt.fileno(), 0o600)
        json.dump(dict(source_revision=CORE_REVISION, ghostty_revision=GHOSTTY_REVISION,
                       version="0.1.22", sha256=args.sha256), receipt)
        receipt.flush()
        os.fsync(receipt.fileno())
    print("Pinned development core staged; no runtime started")


if __name__ == "__main__":
    main()
