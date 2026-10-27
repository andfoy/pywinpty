"""Sanity check that a built sdist contains only project sources before upload."""

import argparse
import glob
import os
import sys
import tarfile

# Directories that must never be packaged. `maturin sdist` archives the working
# tree, so anything a CI step downloads into the checkout gets shipped: the
# published 3.0.5 and 3.1.0 sdists were both ~16 MB because the NuGet
# `pythonarm64` install landed in the repo root (284 of 315 members, 17.9 MB of
# 18.1 MB uncompressed), against ~31 KB for 3.0.3.
FORBIDDEN_PREFIXES = ("pythonarm64", "Microsoft.Windows.Console.ConPTY")

# 3.0.3 was ~31 KB and the tree has not grown materially since, so anything
# approaching a megabyte means something was vendored in by accident.
DEFAULT_MAX_BYTES = 1_000_000


def strip_root(name):
    """Drop the leading `pywinpty-<version>/` component from an archive path."""
    parts = name.split("/", 1)
    return parts[1] if len(parts) > 1 else ""


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("sdist", help="Path or glob to the sdist tarball")
    parser.add_argument(
        "--max-bytes",
        type=int,
        default=DEFAULT_MAX_BYTES,
        help="Fail if the packed tarball exceeds this size (default: %(default)s)",
    )
    args = parser.parse_args()

    matches = sorted(glob.glob(args.sdist))
    if not matches:
        print(f"ERROR: no sdists matched {args.sdist!r}", file=sys.stderr)
        sys.exit(2)

    failed = False
    for path in matches:
        print(f"Checking {path}")
        with tarfile.open(path) as tf:
            members = tf.getmembers()

        packed = os.path.getsize(path)
        unpacked = sum(member.size for member in members)
        print(
            f"  {len(members)} members, {packed} bytes packed, "
            f"{unpacked} bytes unpacked"
        )

        forbidden = sorted(
            {
                strip_root(member.name).split("/")[0]
                for member in members
                if strip_root(member.name).startswith(FORBIDDEN_PREFIXES)
            }
        )
        if forbidden:
            failed = True
            print(f"  FORBIDDEN: {forbidden}", file=sys.stderr)

        if packed > args.max_bytes:
            failed = True
            print(
                f"  TOO LARGE: {packed} bytes packed exceeds {args.max_bytes}",
                file=sys.stderr,
            )

        if forbidden or packed > args.max_bytes:
            print("  Largest members:", file=sys.stderr)
            for member in sorted(members, key=lambda m: -m.size)[:10]:
                print(f"    {member.size:>12} {member.name}", file=sys.stderr)
        else:
            print("  OK")

    if failed:
        sys.exit(1)


if __name__ == "__main__":
    main()
