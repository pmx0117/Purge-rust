import plistlib
import stat
import tarfile
import tempfile
import unittest
import zipfile
from pathlib import Path
from unittest.mock import patch

import release


class ReleaseTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        (self.root / "licenses").mkdir()
        (self.root / "licenses/LICENSE-GPUI").write_text("Apache-2.0")
        (self.root / "README.md").write_text("Purge")
        (self.root / "Cargo.toml").write_text('[workspace.package]\nversion="0.1.0"\n')
        (self.root / "assets").mkdir()
        (self.root / "assets/dev.purge.Purge.desktop").write_text(
            "[Desktop Entry]\nExec=purge\nIcon=dev.purge.Purge\n"
        )
        self.resources = self.root / "build"
        self.resources.mkdir()
        (self.resources / "purge.png").write_bytes(b"PNG fixture")
        (self.resources / "purge.icns").write_bytes(b"ICNS fixture")
        self.binary = self.root / "purge"
        self.binary.write_bytes(b"executable fixture")
        self.binary.chmod(0o755)
        self.output = self.root / "dist"

    def test_tag_must_match_cargo_before_a_release_can_build(self):
        self.assertEqual(release.release_version(self.root, "v0.1.0"), "0.1.0")
        for tag in ["main", "0.1.0", "v0.2.0", "v0.1.0/other"]:
            with self.subTest(tag=tag), self.assertRaises(ValueError):
                release.release_version(self.root, tag)
        (self.root / "Cargo.toml").write_text(
            '[workspace.package]\nversion="0.2.0-rc.1"\n'
        )
        self.assertEqual(
            release.release_version(self.root, "v0.2.0-rc.1"), "0.2.0-rc.1"
        )

    def test_windows_zip_has_an_executable_and_required_licenses(self):
        archive = self.package("x86_64-pc-windows-msvc")
        self.assertEqual(archive.name, "Purge-0.1.0-windows-x64.zip")
        with zipfile.ZipFile(archive) as bundle:
            self.assertEqual(
                bundle.read("Purge-0.1.0-windows-x64/purge.exe"),
                self.binary.read_bytes(),
            )
            self.assertIn(
                "Purge-0.1.0-windows-x64/licenses/LICENSE-GPUI", bundle.namelist()
            )

    def test_linux_archives_keep_executable_permissions_and_desktop_identity(self):
        for target, architecture in [
            ("x86_64-unknown-linux-gnu", "x64"),
            ("aarch64-unknown-linux-gnu", "arm64"),
        ]:
            with self.subTest(target=target):
                archive = self.package(target)
                base = f"Purge-0.1.0-linux-{architecture}"
                with tarfile.open(archive) as bundle:
                    self.assertEqual(
                        bundle.getmember(f"{base}/bin/purge").mode & 0o111, 0o111
                    )
                    entry = bundle.extractfile(
                        f"{base}/share/applications/dev.purge.Purge.desktop"
                    ).read()
                    self.assertIn(b"Icon=dev.purge.Purge", entry)
                    self.assertIn(
                        f"{base}/share/icons/hicolor/256x256/apps/dev.purge.Purge.png",
                        bundle.getnames(),
                    )
                    self.assertIn(
                        f"{base}/share/licenses/purge/LICENSE-GPUI", bundle.getnames()
                    )

    @patch("release.sign_macos_bundle")
    def test_mac_app_bundle_has_version_icon_executable_and_signature_step(self, sign):
        for target in ["aarch64-apple-darwin", "x86_64-apple-darwin"]:
            with self.subTest(target=target):
                archive = self.package(target)
                with zipfile.ZipFile(archive) as bundle:
                    info = plistlib.loads(bundle.read("Purge.app/Contents/Info.plist"))
                    self.assertEqual(info["CFBundleIdentifier"], "dev.purge.Purge")
                    self.assertEqual(info["CFBundleExecutable"], "purge")
                    self.assertEqual(info["CFBundleShortVersionString"], "0.1.0")
                    self.assertEqual(info["CFBundleIconFile"], "purge.icns")
                    executable = bundle.getinfo("Purge.app/Contents/MacOS/purge")
                    self.assertTrue(executable.external_attr >> 16 & stat.S_IXUSR)
                    self.assertIn(
                        "Purge.app/Contents/Resources/licenses/LICENSE-GPUI",
                        bundle.namelist(),
                    )
        self.assertEqual(sign.call_count, 2)

    def test_missing_build_resources_never_create_a_partial_archive(self):
        (self.resources / "purge.png").unlink()
        with self.assertRaises(FileNotFoundError):
            self.package("x86_64-unknown-linux-gnu")
        self.assertFalse(list(self.output.glob("*.tar.gz")))

    def test_cargo_outputs_select_the_application_not_dependency_build_directories(
        self,
    ):
        messages = [
            {
                "reason": "build-script-executed",
                "package_id": "gpui",
                "out_dir": "/wrong",
            },
            {
                "reason": "build-script-executed",
                "package_id": "purge",
                "out_dir": str(self.resources),
            },
            {
                "reason": "compiler-artifact",
                "package_id": "purge",
                "executable": str(self.binary),
            },
        ]
        self.assertEqual(
            release.build_outputs(messages, "purge"), (self.binary, self.resources)
        )
        with self.assertRaises(ValueError):
            release.build_outputs(messages, "missing")

    def test_checksums_require_every_platform_and_match_the_downloaded_bytes(self):
        self.output.mkdir()
        expected = release.archive_names("0.1.0")
        for name in expected[:-1]:
            (self.output / name).write_bytes(name.encode())
        with self.assertRaises(ValueError):
            release.write_checksums(self.output, "0.1.0")
        (self.output / expected[-1]).write_bytes(expected[-1].encode())
        manifest = release.write_checksums(self.output, "0.1.0")
        import hashlib

        entries = manifest.read_text().splitlines()
        self.assertEqual(len(entries), len(expected))
        for line in entries:
            digest, name = line.split("  ", 1)
            self.assertEqual(
                digest, hashlib.sha256((self.output / name).read_bytes()).hexdigest()
            )

    def package(self, target):
        return release.package_release(
            self.root, target, self.binary, self.resources, self.output, "0.1.0"
        )


if __name__ == "__main__":
    unittest.main()
