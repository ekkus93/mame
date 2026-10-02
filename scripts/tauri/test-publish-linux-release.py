#!/usr/bin/env python3
"""Exercise release provenance checks without network access or release mutations."""
import importlib.util
from pathlib import Path
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location(
    "publisher", Path(__file__).with_name("publish-linux-release.py")
)
publisher = importlib.util.module_from_spec(spec)
spec.loader.exec_module(publisher)


class PublisherTests(unittest.TestCase):
    def exercise(self, case):
        sha = "a" * 40
        run = {
            "head_branch": "v0.1.0", "name": "Tauri Linux real runtime package",
            "event": "push", "status": "completed", "conclusion": "success",
            "head_repository": {"full_name": "ekkus93/mame"}, "head_sha": sha,
            "html_url": "https://github.com/ekkus93/mame/actions/runs/123",
        }
        artifact = {"name": "real-bundled-mame-linux-packages-v0.1.0", "expired": False}
        artifacts = [artifact]
        calls = []
        if case == "wrong_sha":
            run["head_sha"] = "b" * 40
        elif case == "failed":
            run["conclusion"] = "failure"
        elif case == "wrong_repo":
            run["head_repository"]["full_name"] = "attacker/mame"
        elif case == "branch":
            run["head_branch"] = "master"
        elif case == "expired":
            artifact["expired"] = True
        elif case == "duplicates":
            artifacts.append(dict(artifact))

        def api(path):
            if path.endswith("/123"):
                return run
            if "/git/ref/" in path:
                return {"object": {"type": "tag" if case == "annotated" else "commit", "sha": sha}}
            if "/git/tags/" in path:
                return {"object": {"type": "commit", "sha": sha}}
            if "/artifacts?" in path:
                return {"artifacts": artifacts}
            return []

        def gh(*args):
            calls.append(args)
            if args[:2] == ("run", "download"):
                root = Path(args[args.index("--dir") + 1])
                (root / "mame.deb").write_bytes(b"deb")
                if case != "missing_package":
                    (root / "mame.AppImage").write_bytes(b"appimage")
            return ""

        with patch.object(publisher, "api", api), patch.object(publisher, "gh", gh):
            if case in ("success", "annotated"):
                publisher.publish("ekkus93/mame", "123")
                self.assertTrue(any(c[:2] == ("release", "upload") for c in calls))
                self.assertTrue(any(c[:2] == ("release", "edit") for c in calls))
            else:
                with self.assertRaises(ValueError):
                    publisher.publish("ekkus93/mame", "123")
                self.assertFalse(any(c[0] == "release" for c in calls))

    def test_valid_lightweight_and_annotated_tags(self):
        for case in ("success", "annotated"):
            with self.subTest(case=case):
                self.exercise(case)

    def test_reject_unqualified_or_incomplete_packages_before_release_write(self):
        for case in ("wrong_sha", "failed", "wrong_repo", "branch", "expired",
                     "duplicates", "missing_package"):
            with self.subTest(case=case):
                self.exercise(case)


if __name__ == "__main__":
    unittest.main()
