import contextlib
import io
import os
import plistlib
import stat
import subprocess
import sys
import tarfile
import tempfile
import unittest
import zipfile
from functools import partial
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

    def test_invalid_workspace_version_is_rejected(self):
        (self.root / "Cargo.toml").write_text('[workspace.package]\nversion="broken"\n')
        with self.assertRaises(ValueError):
            release.release_version(self.root)

    def test_cli_rejects_missing_release_command_tag_or_target(self):
        for argv in [
            ["release.py"],
            ["release.py", "version"],
            ["release.py", "build"],
        ]:
            with (
                self.subTest(argv=argv),
                patch("sys.argv", argv),
                contextlib.redirect_stderr(io.StringIO()),
                self.assertRaises(SystemExit) as error,
            ):
                release.main()
            self.assertEqual(error.exception.code, 2)

    def test_version_cli_marks_prereleases_and_writes_workflow_outputs(self):
        (self.root / "Cargo.toml").write_text(
            '[workspace.package]\nversion="0.2.0-rc.1"\n'
        )
        output = self.root / "github-output"
        argv = [
            "release.py",
            "version",
            "--tag",
            "v0.2.0-rc.1",
            "--github-output",
            str(output),
        ]
        with (
            patch.object(release, "ROOT", self.root),
            patch("sys.argv", argv),
            contextlib.redirect_stdout(io.StringIO()),
        ):
            release.main()
        self.assertEqual(
            output.read_text(), "version=0.2.0-rc.1\ntag=v0.2.0-rc.1\nprerelease=true\n"
        )

    def test_build_cli_packages_the_selected_native_executable(self):
        argv = [
            "release.py",
            "build",
            "--target",
            "x86_64-pc-windows-msvc",
            "--output",
            str(self.output),
        ]
        with (
            patch.object(release, "ROOT", self.root),
            patch("sys.argv", argv),
            patch("release.build_release", return_value=(self.binary, self.resources)),
            patch(
                "release.compile_windows_installer", side_effect=self.installer_probe
            ),
            contextlib.redirect_stdout(io.StringIO()),
        ):
            release.main()
        self.assertEqual(
            (self.output / "Purge-0.1.0-windows-x64-setup.exe").read_bytes(),
            b"MZ installer fixture",
        )

    def test_checksums_cli_refuses_to_release_an_incomplete_platform_set(self):
        self.output.mkdir()
        argv = ["release.py", "checksums", "--directory", str(self.output)]
        with (
            patch.object(release, "ROOT", self.root),
            patch("sys.argv", argv),
            self.assertRaises(ValueError),
        ):
            release.main()

    @patch("release.compile_windows_installer")
    def test_windows_emits_a_setup_exe_with_the_application_and_licenses(
        self, compiler
    ):
        compiler.side_effect = self.installer_probe
        installer = self.package("x86_64-pc-windows-msvc")
        self.assertEqual(installer.name, "Purge-0.1.0-windows-x64-setup.exe")
        self.assertEqual(installer.read_bytes(), b"MZ installer fixture")
        self.assertFalse(list(self.output.glob("*.zip")))

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
            {"reason": "compiler-artifact", "package_id": "purge", "executable": None},
        ]
        self.assertEqual(
            release.build_outputs(messages, "purge"), (self.binary, self.resources)
        )
        with self.assertRaises(ValueError):
            release.build_outputs(messages, "missing")
        for incomplete in [messages[1:2], messages[2:3]]:
            with self.subTest(messages=incomplete), self.assertRaises(ValueError):
                release.build_outputs(incomplete, "purge")

    def test_packaging_can_create_a_nested_output_directory(self):
        self.output = self.root / "nested/output/dist"
        archive = self.package("x86_64-unknown-linux-gnu")
        self.assertTrue(tarfile.is_tarfile(archive))

    def test_installer_only_matrix_selects_exactly_the_windows_native_runner(self):
        self.assertEqual(
            release.build_matrix(windows_only=True),
            {
                "include": [
                    {
                        "name": "Windows x64",
                        "runner": "windows-2022",
                        "target": "x86_64-pc-windows-msvc",
                    }
                ]
            },
        )
        self.assertEqual(len(release.build_matrix()["include"]), 5)

    def test_compiler_failure_never_reuses_a_stale_installer(self):
        self.output.mkdir()
        destination = self.output / "Purge-0.1.0-windows-x64-setup.exe"
        destination.write_bytes(b"stale installer")
        with (
            patch.dict(os.environ, {"ISCC_PATH": sys.executable}),
            patch(
                "release.subprocess.run",
                partial(subprocess.run, stderr=subprocess.PIPE),
            ),
            self.assertRaises(subprocess.CalledProcessError),
        ):
            release.compile_windows_installer(
                self.root, self.root, self.resources, destination, "0.1.0"
            )
        self.assertFalse(destination.exists())

    def test_compiler_creates_the_first_installer_in_a_fresh_output_directory(self):
        self.output.mkdir()
        destination = self.output / "Purge-0.1.0-windows-x64-setup.exe"
        compiler = self.root / "ISCC.exe"
        compiler.write_bytes(b"compiler fixture")
        with (
            patch.dict(os.environ, {"ISCC_PATH": str(compiler)}),
            patch(
                "release.subprocess.run",
                side_effect=lambda *args, **kwargs: destination.write_bytes(
                    b"MZ setup"
                ),
            ),
        ):
            release.compile_windows_installer(
                self.root, self.root, self.resources, destination, "0.1.0"
            )
        self.assertEqual(destination.read_bytes(), b"MZ setup")

    def test_matrix_cli_exposes_only_windows_for_draft_repack(self):
        output = self.root / "matrix-output"
        argv = [
            "release.py",
            "matrix",
            "--windows-only",
            "--github-output",
            str(output),
        ]
        with patch("sys.argv", argv), contextlib.redirect_stdout(io.StringIO()):
            release.main()
        self.assertTrue(output.read_text().startswith('matrix={"include":'))
        self.assertNotIn("apple", output.read_text())
        self.assertNotIn("linux", output.read_text())

    def test_updating_windows_instructions_preserves_custom_release_notes(self):
        template = self.root / ".github/release-notes.md"
        template.parent.mkdir(parents=True)
        template.write_text("| Windows x64 | `windows-x64-setup.exe` | 双击安装 |\n")
        notes = self.root / "draft.md"
        notes.write_text(
            "我的说明\n| Windows x64 | `windows-x64.zip` | 解压运行 |\n其他平台与更新记录\n"
        )
        release.update_windows_notes(self.root, notes)
        expected = "我的说明\n| Windows x64 | `windows-x64-setup.exe` | 双击安装 |\n其他平台与更新记录\n"
        self.assertEqual(notes.read_text(), expected)
        release.update_windows_notes(self.root, notes)
        self.assertEqual(notes.read_text(), expected)

    def installer_probe(self, root, staging, resources, destination, version):
        self.assertEqual((staging / "purge.exe").read_bytes(), self.binary.read_bytes())
        self.assertEqual((staging / "licenses/LICENSE-GPUI").read_text(), "Apache-2.0")
        self.assertEqual(version, "0.1.0")
        destination.write_bytes(b"MZ installer fixture")

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
