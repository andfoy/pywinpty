"""Sanity check that a built wheel contains every expected binary before upload."""

import argparse
import fnmatch
import glob
import os
import sys
import zipfile


def list_wheel(wheel_path):
    with zipfile.ZipFile(wheel_path) as zf:
        return zf.namelist()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("wheel", help="Path or glob to the wheel file")
    parser.add_argument(
        "--has-conpty",
        action="store_true",
        help="Require ConPTY runtime binaries (conpty.dll, OpenConsole.exe)",
    )
    parser.add_argument(
        "--has-winpty",
        action="store_true",
        help="Require WinPTY runtime binaries (winpty.dll, winpty-agent.exe)",
    )
    abi = parser.add_mutually_exclusive_group()
    abi.add_argument(
        "--abi3",
        action="store_true",
        help="Require a stable-ABI (abi3) wheel",
    )
    abi.add_argument(
        "--no-abi3",
        action="store_true",
        help="Require a version-specific (non-abi3) wheel, e.g. free-threaded",
    )
    parser.add_argument(
        "--extension-pattern",
        # abi3 builds name the extension `_winpty.pyd`, version-specific
        # builds `_winpty.cp3XX-win_*.pyd`.
        default="winpty/_winpty*.pyd",
        help="Glob the rust extension file must match",
    )
    args = parser.parse_args()

    matches = sorted(glob.glob(args.wheel))
    if not matches:
        print(f"ERROR: no wheels matched {args.wheel!r}", file=sys.stderr)
        sys.exit(2)

    failed = False
    for wheel_path in matches:
        print(f"Checking {wheel_path}")
        names = list_wheel(wheel_path)

        expected = []
        if args.has_conpty:
            expected += ["winpty/conpty.dll", "winpty/OpenConsole.exe"]
        if args.has_winpty:
            expected += ["winpty/winpty.dll", "winpty/winpty-agent.exe"]

        missing = [name for name in expected if name not in names]
        ext_match = any(fnmatch.fnmatch(n, args.extension_pattern) for n in names)
        if not ext_match:
            missing.append(args.extension_pattern)

        # Wheel filename: {name}-{version}-{python tag}-{abi tag}-{platform}.whl
        abi_tag = os.path.basename(wheel_path)[:-len(".whl")].split("-")[-2]
        if args.abi3 and abi_tag != "abi3":
            missing.append(f"abi3 tag (got {abi_tag!r})")
        if args.no_abi3 and abi_tag == "abi3":
            missing.append("version-specific abi tag (got 'abi3')")

        if missing:
            failed = True
            print(f"  MISSING: {missing}", file=sys.stderr)
            print("  Wheel contents:", file=sys.stderr)
            for n in names:
                print(f"    {n}", file=sys.stderr)
        else:
            print("  OK")

    if failed:
        sys.exit(1)


if __name__ == "__main__":
    main()
