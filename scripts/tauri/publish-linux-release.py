#!/usr/bin/env python3
"""Promote a successful tagged package run without rebuilding or moving its tag."""
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import tempfile


def gh(*args):
    return subprocess.check_output(["gh", *args], text=True)


def api(path):
    return json.loads(gh("api", path))


def digest(path):
    checksum = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            checksum.update(chunk)
    return checksum.hexdigest()


def publish(repo, run_id):
    if not re.fullmatch(r"[0-9]+", run_id):
        raise ValueError("Package run ID must be numeric")
    run = api(f"repos/{repo}/actions/runs/{run_id}")
    tag = run["head_branch"]
    if not re.fullmatch(r"v[0-9]+\.[0-9]+\.[0-9]+(?:[-+][A-Za-z0-9.-]+)?", tag):
        raise ValueError("Only version-tagged package runs may be published")
    if (run["name"] != "Tauri Linux real runtime package"
            or run["event"] not in ("push", "workflow_dispatch")
            or run["status"] != "completed" or run["conclusion"] != "success"
            or run["head_repository"]["full_name"] != repo):
        raise ValueError("Run is not a successful trusted real-runtime package build")
    tag_sha = subprocess.check_output(
        ["git", "rev-parse", f"refs/tags/{tag}^{{commit}}"], text=True
    ).strip()
    if tag_sha != run["head_sha"]:
        raise ValueError("Package run SHA differs from the existing version tag")
    artifacts = api(f"repos/{repo}/actions/runs/{run_id}/artifacts?per_page=100")["artifacts"]
    name = f"real-bundled-mame-linux-packages-{tag}"
    matches = [a for a in artifacts if a["name"] == name and not a["expired"]]
    if len(matches) != 1:
        raise ValueError("Exactly one unexpired tagged package artifact is required")

    with tempfile.TemporaryDirectory(prefix="mame-release-") as directory:
        root = Path(directory)
        gh("run", "download", run_id, "--repo", repo, "--name", name, "--dir", str(root))
        packages = sorted(p for p in root.rglob("*") if p.is_file() and p.suffix in (".deb", ".AppImage"))
        if (sum(p.suffix == ".deb" for p in packages) != 1
                or sum(p.suffix == ".AppImage" for p in packages) != 1):
            raise ValueError("Artifact must contain one Debian package and one AppImage")
        if any(not re.fullmatch(r"[A-Za-z0-9_.+-]+", p.name) for p in packages):
            raise ValueError("Package filenames contain unsupported characters")
        sums = root / "SHA256SUMS"
        sums.write_text("".join(
            f"{digest(p)}  {p.name}\n"
            for p in packages
        ), encoding="utf-8")
        notes = root / "release-notes.md"
        limitation = (
            "\nKnown limitation: this original v0.1.0 build bundles MAME's tiny example "
            "driver set. It does not support general arcade ROM collections. "
            "Subsequent release builds require the full MAME runtime.\n"
            if tag == "v0.1.0" else ""
        )
        notes.write_text(
            f"Linux Debian package and AppImage, including the bundled MAME runtime.\n\n"
            f"Source commit: `{tag_sha}`\n\n"
            f"Package qualification: {run['html_url']}\n\n"
            "These are persistent release assets; Actions artifact retention does not apply.\n"
            + limitation, encoding="utf-8"
        )
        listed = api(f"repos/{repo}/releases?per_page=100")
        existing = next((release for release in listed if release["tag_name"] == tag), None)
        if existing is None:
            gh("release", "create", tag, "--repo", repo, "--verify-tag", "--draft",
               "--title", tag, "--notes-file", str(notes))
        else:
            # Never silently replace assets from an already published version.
            for asset in existing["assets"]:
                matching = next((p for p in packages + [sums] if p.name == asset["name"]), None)
                if matching:
                    remote = root / "existing"
                    remote.mkdir(exist_ok=True)
                    gh("release", "download", tag, "--repo", repo, "--pattern", matching.name,
                       "--dir", str(remote))
                    if (remote / matching.name).read_bytes() != matching.read_bytes():
                        raise ValueError(f"Existing release asset differs: {matching.name}")
        present = {a["name"] for a in existing["assets"]} if existing else set()
        missing = [str(p) for p in packages + [sums] if p.name not in present]
        if missing:
            gh("release", "upload", tag, *missing, "--repo", repo)
        if existing is None or existing["draft"]:
            gh("release", "edit", tag, "--repo", repo, "--draft=false")
        print(f"Published https://github.com/{repo}/releases/tag/{tag}")


if __name__ == "__main__":
    publish(os.environ["GH_REPO"], os.environ["PACKAGE_RUN_ID"])
