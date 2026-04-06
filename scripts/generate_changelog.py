#!/usr/bin/env python3
"""Generate changelog.json from git tags and commit messages.

Usage:
    python3 scripts/generate_changelog.py [--output desktop/public/changelog.json]

Reads git log grouped by version tags and outputs a JSON array:
[
  { "version": "0.1.23", "date": "2026-04-04", "changes": ["feat: ...", "fix: ..."] },
  ...
]
"""

import subprocess
import json
import sys
import os
import re

def run(cmd):
    result = subprocess.run(cmd, capture_output=True, text=True, cwd=os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
    return result.stdout.strip()

def get_tags():
    """Get all version tags sorted by date descending."""
    raw = run(["git", "tag", "--sort=-creatordate"])
    if not raw:
        return []
    return [t.strip() for t in raw.splitlines() if re.match(r"^v?\d+\.\d+\.\d+", t.strip())]

def get_tag_date(tag):
    """Get the date of a tag in YYYY-MM-DD format."""
    date_str = run(["git", "log", "-1", "--format=%ai", tag])
    if date_str:
        return date_str[:10]
    return ""

def get_commits_between(from_ref, to_ref):
    """Get commit messages between two refs."""
    if from_ref:
        range_spec = f"{from_ref}..{to_ref}"
    else:
        range_spec = to_ref
    raw = run(["git", "log", range_spec, "--pretty=format:%s", "--no-merges"])
    if not raw:
        return []
    lines = [line.strip() for line in raw.splitlines() if line.strip()]
    # Filter out noise (version bumps, skip-ci, auto-sync)
    skip_patterns = [r"^\[skip ci\]", r"^chore: auto-bump", r"^auto: sync", r"^Merge "]
    filtered = []
    for line in lines:
        if any(re.search(p, line, re.IGNORECASE) for p in skip_patterns):
            continue
        filtered.append(line)
    return filtered

def get_head_commits_since_tag(tag):
    """Get commits from tag to HEAD (unreleased)."""
    return get_commits_between(tag, "HEAD")

def normalize_version(tag):
    """Strip leading 'v' from tag name."""
    return tag.lstrip("v")

def main():
    output_path = "desktop/public/changelog.json"
    if len(sys.argv) > 2 and sys.argv[1] == "--output":
        output_path = sys.argv[2]

    tags = get_tags()
    releases = []

    # Unreleased changes (HEAD since latest tag)
    if tags:
        unreleased = get_head_commits_since_tag(tags[0])
        if unreleased:
            current_version = run(["git", "describe", "--tags", "--abbrev=0", "HEAD"]) or tags[0]
            # Read version from Cargo.toml as it might be bumped but not tagged yet
            cargo_toml = os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), "Cargo.toml")
            cargo_version = ""
            if os.path.exists(cargo_toml):
                with open(cargo_toml) as f:
                    for line in f:
                        m = re.match(r'^version\s*=\s*"([^"]+)"', line)
                        if m:
                            cargo_version = m.group(1)
                            break
            version = cargo_version or normalize_version(current_version)
            releases.append({
                "version": version,
                "date": run(["git", "log", "-1", "--format=%ai", "HEAD"])[:10],
                "changes": unreleased
            })

    # Tagged releases
    for i, tag in enumerate(tags):
        prev_tag = tags[i + 1] if i + 1 < len(tags) else None
        changes = get_commits_between(prev_tag, tag)
        if not changes:
            continue
        releases.append({
            "version": normalize_version(tag),
            "date": get_tag_date(tag),
            "changes": changes
        })

    # Limit to last 20 releases
    releases = releases[:20]

    # Write output
    os.makedirs(os.path.dirname(os.path.abspath(output_path)), exist_ok=True)
    with open(output_path, "w", encoding="utf-8") as f:
        json.dump(releases, f, ensure_ascii=False, indent=2)

    print(f"Generated {len(releases)} releases → {output_path}")

if __name__ == "__main__":
    main()
