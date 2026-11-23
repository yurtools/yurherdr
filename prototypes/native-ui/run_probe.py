#!/usr/bin/env python3
"""Launch only this guarded standalone probe with its own process profile.

Preserves normal HOME and native display endpoints. Never invokes hc, old-app
helpers, installed/live cores or service managers. Caller must stage a verified
core in a fresh marked run first. Closing views keeps the run alive; Quit stops.
"""
import argparse
import os
from pathlib import Path
import subprocess


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--probe", type=Path, required=True)
    parser.add_argument("--run-root", type=Path, required=True)
    parser.add_argument("--nonce", required=True)
    parser.add_argument("--smoke", action="store_true", help="quit after five seconds")
    args = parser.parse_args()
    if not args.probe.is_absolute() or not args.run_root.is_absolute():
        parser.error("use absolute owned probe/run paths")
    if "HC_SESSION" in os.environ:
        parser.error("launch outside the inherited live core session")
    subprocess.run([str(args.probe), "--guard-only", str(args.run_root), "--nonce", args.nonce], check=True)
    # No parent/live profile variable is changed. The new process receives only
    # its native display inputs plus the explicitly owned evaluation profile.
    environment = {name: os.environ[name] for name in ["HOME", "DISPLAY", "WAYLAND_DISPLAY",
        "XDG_RUNTIME_DIR", "XAUTHORITY", "LANG", "LC_ALL", "LC_CTYPE", "XMODIFIERS",
        "GTK_IM_MODULE", "QT_IM_MODULE"] if name in os.environ}
    for variable, directory in [("XDG_CONFIG_HOME", "config"), ("XDG_DATA_HOME", "data"),
        ("XDG_CACHE_HOME", "cache"), ("XDG_STATE_HOME", "data"),
        ("YURHERDR_CONFIG_DIR", "config"), ("YURHERDR_DATA_DIR", "data"),
        ("YURHERDR_CACHE_DIR", "cache"), ("YURHERDR_LOG_DIR", "logs")]:
        environment[variable] = str(args.run_root / directory)
    try:
        result = subprocess.run([str(args.probe), "--native-smoke" if args.smoke else "--native",
            str(args.run_root), "--nonce", args.nonce], env=environment, cwd=args.run_root / "data",
            timeout=20 if args.smoke else None)
    except subprocess.TimeoutExpired:
        # subprocess.run kills/reaps only this owned probe. Linux fixture/core
        # parent-death signals cover abrupt probe exit, not just normal Quit.
        print("Yurherdr smoke exceeded twenty seconds; owned probe stopped")
        raise SystemExit(124)
    raise SystemExit(result.returncode)


if __name__ == "__main__":
    main()
