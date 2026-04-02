#!/usr/bin/env python3

import json
import re
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]


def read_root_version() -> str:
    cargo_toml = (ROOT / "Cargo.toml").read_text(encoding="utf-8")
    match = re.search(r'^version = "([^"]+)"', cargo_toml, re.MULTILINE)
    if not match:
        raise SystemExit("Failed to read version from Cargo.toml")
    return match.group(1)


def write_json(path: Path, update):
    data = json.loads(path.read_text(encoding="utf-8"))
    update(data)
    path.write_text(json.dumps(data, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def replace_first_version(path: Path, version: str):
    content = path.read_text(encoding="utf-8")
    updated, count = re.subn(
        r'^version = "[^"]+"',
        f'version = "{version}"',
        content,
        count=1,
        flags=re.MULTILINE,
    )
    if count != 1:
        raise SystemExit(f"Failed to update version in {path}")
    path.write_text(updated, encoding="utf-8")


def main():
    version = sys.argv[1] if len(sys.argv) > 1 else read_root_version()

    write_json(ROOT / "desktop" / "package.json", lambda data: data.__setitem__("version", version))
    write_json(ROOT / "desktop" / "src-tauri" / "tauri.conf.json", lambda data: data.__setitem__("version", version))
    replace_first_version(ROOT / "desktop" / "src-tauri" / "Cargo.toml", version)

    print(f"Synchronized desktop versions to {version}")


if __name__ == "__main__":
    main()
