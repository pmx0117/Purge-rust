"""Build native release bundles using Cargo's exact executable and resource outputs."""

import argparse
import hashlib
import json
import os
import plistlib
import re
import shutil
import subprocess
import tarfile
import tempfile
import tomllib
import zipfile
from pathlib import Path
from typing import NamedTuple


class Platform(NamedTuple):
    system: str
    architecture: str
    suffix: str
    runner: str
    name: str


ROOT = Path(__file__).resolve().parents[2]
PLATFORMS = {
    "x86_64-pc-windows-msvc": Platform(
        "windows", "x64", "-setup.exe", "windows-2022", "Windows x64"
    ),
    "x86_64-unknown-linux-gnu": Platform(
        "linux", "x64", ".tar.gz", "ubuntu-24.04", "Linux x64"
    ),
    "aarch64-unknown-linux-gnu": Platform(
        "linux", "arm64", ".tar.gz", "ubuntu-24.04-arm", "Linux ARM64"
    ),
    "x86_64-apple-darwin": Platform(
        "macos", "x64", ".zip", "macos-15-intel", "macOS Intel"
    ),
    "aarch64-apple-darwin": Platform(
        "macos", "arm64", ".zip", "macos-15", "macOS Apple Silicon"
    ),
}


def release_version(root, tag=None):
    with (root / "Cargo.toml").open("rb") as source:
        version = tomllib.load(source)["workspace"]["package"]["version"]
    if not re.fullmatch(
        r"\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?(?:\+[0-9A-Za-z.-]+)?", version
    ):
        raise ValueError(f"Invalid release version: {version}")
    if tag is not None and tag != f"v{version}":
        raise ValueError(f"Tag {tag!r} does not match Cargo version v{version}")
    return version


def archive_name(version, target):
    platform = PLATFORMS[target]
    return f"Purge-{version}-{platform.system}-{platform.architecture}{platform.suffix}"


def archive_names(version):
    return [archive_name(version, target) for target in PLATFORMS]


def build_matrix(windows_only=False):
    return {
        "include": [
            {"name": platform.name, "runner": platform.runner, "target": target}
            for target, platform in PLATFORMS.items()
            if not windows_only or platform.system == "windows"
        ]
    }


def build_outputs(messages, package_id):
    binary = resources = None
    for message in messages:
        if message.get("package_id") != package_id:
            continue
        if message["reason"] == "build-script-executed":
            resources = Path(message["out_dir"])
        if message["reason"] == "compiler-artifact" and message.get("executable"):
            binary = Path(message["executable"])
    if binary is None or resources is None:
        raise ValueError(
            "Cargo did not produce the Purge executable and build resources"
        )
    return binary, resources


def build_release(target):
    metadata = subprocess.check_output(
        ["cargo", "metadata", "--locked", "--no-deps", "--format-version", "1"],
        cwd=ROOT,
        text=True,
        encoding="utf-8",
    )
    package = next(p for p in json.loads(metadata)["packages"] if p["name"] == "purge")
    command = [
        "cargo",
        "build",
        "--locked",
        "--release",
        "-p",
        "purge",
        "--target",
        target,
        "--message-format=json-render-diagnostics",
    ]
    with subprocess.Popen(
        command, cwd=ROOT, stdout=subprocess.PIPE, text=True, encoding="utf-8"
    ) as process:
        messages = [json.loads(line) for line in process.stdout]
        if process.wait() != 0:
            raise subprocess.CalledProcessError(process.returncode, command)
    return build_outputs(messages, package["id"])


def copy_notices(root, destination):
    shutil.copytree(root / "licenses", destination / "licenses")
    shutil.copy2(root / "README.md", destination / "README.md")


def windows_bundle(root, staging, binary, resources, version):
    shutil.copy2(binary, staging / "purge.exe")
    copy_notices(root, staging)


def inno_setup_compiler():
    compiler = Path(
        os.environ.get("ISCC_PATH")
        or (
            Path(os.environ.get("ProgramFiles(x86)", r"C:\Program Files (x86)"))
            / "Inno Setup 6/ISCC.exe"
        )
    )
    if not compiler.is_file():
        raise FileNotFoundError("Install Inno Setup 6.7+ or set ISCC_PATH to ISCC.exe")
    return compiler


def compile_windows_installer(root, staging, resources, destination, version):
    destination.unlink(missing_ok=True)
    numeric_version = re.split(r"[-+]", version)[0] + ".0"
    command = [
        str(inno_setup_compiler()),
        "/Qp",
        f"/DAppVersion={version}",
        f"/DAppNumericVersion={numeric_version}",
        f"/DSourceDir={staging.resolve()}",
        f"/DIconFile={(resources / 'purge.ico').resolve()}",
        f"/O{destination.parent.resolve()}",
        f"/F{destination.stem}",
        str(root / ".github/installer/windows.iss"),
    ]
    subprocess.run(command, check=True)
    if not destination.is_file():
        raise FileNotFoundError("Inno Setup did not produce the requested installer")


def linux_bundle(root, staging, binary, resources, version):
    executable = staging / "bin/purge"
    executable.parent.mkdir()
    shutil.copy2(binary, executable)
    executable.chmod(0o755)
    applications = staging / "share/applications"
    applications.mkdir(parents=True)
    shutil.copy2(root / "assets/dev.purge.Purge.desktop", applications)
    icons = staging / "share/icons/hicolor/256x256/apps"
    icons.mkdir(parents=True)
    shutil.copy2(resources / "purge.png", icons / "dev.purge.Purge.png")
    notices = staging / "share/licenses/purge"
    notices.mkdir(parents=True)
    shutil.copytree(root / "licenses", notices, dirs_exist_ok=True)
    shutil.copy2(root / "README.md", staging / "README.md")


def macos_bundle(root, staging, binary, resources, version):
    contents = staging / "Contents"
    executables = contents / "MacOS"
    executables.mkdir(parents=True)
    executable = executables / "purge"
    shutil.copy2(binary, executable)
    executable.chmod(0o755)
    bundle_resources = contents / "Resources"
    bundle_resources.mkdir()
    shutil.copy2(resources / "purge.icns", bundle_resources)
    copy_notices(root, bundle_resources)
    numeric_version = re.split(r"[-+]", version)[0]
    info = {
        "CFBundleName": "Purge",
        "CFBundleDisplayName": "Purge",
        "CFBundleIdentifier": "dev.purge.Purge",
        "CFBundleExecutable": "purge",
        "CFBundlePackageType": "APPL",
        "CFBundleShortVersionString": numeric_version,
        "CFBundleVersion": numeric_version,
        "CFBundleGetInfoString": f"Purge {version}",
        "CFBundleIconFile": "purge.icns",
        "LSMinimumSystemVersion": "11.0",
        "NSHighResolutionCapable": True,
        "NSPrincipalClass": "NSApplication",
    }
    with (contents / "Info.plist").open("wb") as output:
        plistlib.dump(info, output)
    sign_macos_bundle(staging)


def sign_macos_bundle(bundle):
    subprocess.run(
        [
            "codesign",
            "--force",
            "--deep",
            "--sign",
            "-",
            "--timestamp=none",
            str(bundle),
        ],
        check=True,
    )
    subprocess.run(
        ["codesign", "--verify", "--deep", "--strict", str(bundle)], check=True
    )


BUNDLERS = {"windows": windows_bundle, "linux": linux_bundle, "macos": macos_bundle}


def write_archive(staging, destination):
    if destination.name.endswith(".tar.gz"):
        with tarfile.open(destination, "w:gz") as archive:
            archive.add(staging, arcname=staging.name)
    else:
        with zipfile.ZipFile(destination, "w", zipfile.ZIP_DEFLATED) as archive:
            for file in sorted(staging.rglob("*")):
                if file.is_file():
                    archive.write(file, file.relative_to(staging.parent))


def package_release(root, target, binary, resources, output, version):
    platform = PLATFORMS[target]
    name = f"Purge-{version}-{platform.system}-{platform.architecture}"
    with tempfile.TemporaryDirectory(prefix="purge-package-") as temporary:
        staging = Path(temporary) / (
            "Purge.app" if platform.system == "macos" else name
        )
        staging.mkdir()
        BUNDLERS[platform.system](root, staging, binary, resources, version)
        output.mkdir(parents=True, exist_ok=True)
        destination = output / archive_name(version, target)
        if platform.system == "windows":
            compile_windows_installer(root, staging, resources, destination, version)
        else:
            write_archive(staging, destination)
    return destination


def sha256_file(path):
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def write_checksums(directory, version):
    expected = set(archive_names(version))
    actual = {
        p.name
        for p in directory.iterdir()
        if p.name.endswith((".exe", ".zip", ".tar.gz"))
    }
    if actual != expected:
        raise ValueError(
            f"Release artifacts differ: missing={sorted(expected - actual)}, unexpected={sorted(actual - expected)}"
        )
    manifest = directory / "SHA256SUMS"
    manifest.write_text(
        "".join(
            f"{sha256_file(directory / name)}  {name}\n" for name in sorted(expected)
        ),
        encoding="utf-8",
        newline="\n",
    )
    return manifest


def emit_workflow_outputs(values, path):
    if path:
        with path.open("a", encoding="utf-8") as output:
            for key, value in values.items():
                output.write(f"{key}={value}\n")
    print(json.dumps(values))


def version_command(args):
    version = release_version(ROOT, args.tag)
    emit_workflow_outputs(
        {
            "version": version,
            "tag": f"v{version}",
            "prerelease": str("-" in version.split("+", 1)[0]).lower(),
        },
        args.github_output,
    )


def update_windows_notes(root, notes):
    template = (root / ".github/release-notes.md").read_text(encoding="utf-8")
    row = next(
        line for line in template.splitlines() if line.startswith("| Windows x64 |")
    )
    body, count = re.subn(
        r"(?m)^\| Windows x64 \|[^\r\n]*", row, notes.read_text(encoding="utf-8")
    )
    if count != 1:
        raise ValueError("Expected one Windows row in the existing draft notes")
    notes.write_text(body, encoding="utf-8", newline="\n")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    version = commands.add_parser("version")
    version.add_argument("--tag", required=True)
    version.add_argument("--github-output", type=Path)
    build = commands.add_parser("build")
    build.add_argument("--target", choices=PLATFORMS, required=True)
    build.add_argument("--output", type=Path, default=ROOT / "dist")
    checksums = commands.add_parser("checksums")
    checksums.add_argument("--directory", type=Path, default=ROOT / "dist")
    matrix = commands.add_parser("matrix")
    matrix.add_argument("--windows-only", action="store_true")
    matrix.add_argument("--github-output", type=Path)
    notes = commands.add_parser("windows-notes")
    notes.add_argument("file", type=Path)
    args = parser.parse_args()
    if args.command == "version":
        version_command(args)
    elif args.command == "build":
        binary, resources = build_release(args.target)
        print(
            package_release(
                ROOT, args.target, binary, resources, args.output, release_version(ROOT)
            )
        )
    elif args.command == "checksums":
        print(write_checksums(args.directory, release_version(ROOT)))
    elif args.command == "matrix":
        emit_workflow_outputs(
            {"matrix": json.dumps(build_matrix(args.windows_only))}, args.github_output
        )
    else:
        update_windows_notes(ROOT, args.file)


if __name__ == "__main__":
    main()
