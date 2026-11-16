#!/usr/bin/env python3
"""Build a native Sinesi package from the locked Cargo release binaries."""
import argparse
import os
from pathlib import Path
import platform
import plistlib
import shutil
import subprocess
import tempfile
import tomllib

PACKAGING = Path(__file__).resolve().parent
ROOT = PACKAGING.parent
PROJECT_MIME = "application/vnd.sinesi.project+json"
PROJECT_UTI = "fr.sinesi.project"
APP_ID = "fr.sinesi.Sinesi"


def run(*args):
    subprocess.run(args, check=True)


def copy(source, destination, *, executable=False):
    destination.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(source, destination)
    destination.chmod(0o755 if executable else 0o644)


def linux(bin_dir, stage, destination, version, arch):
    machine = {"amd64": 62, "arm64": 183}[arch]
    for name in ("sns-desktop", "sinesi"):
        binary = bin_dir / name
        with binary.open("rb") as stream:
            header = stream.read(20)
        if header[:6] != b"\x7fELF\x02\x01" or int.from_bytes(header[18:20], "little") != machine:
            raise SystemExit(f"Expected a Linux {arch} ELF binary: {binary}")
    root = stage / "deb"
    control = root / "DEBIAN" / "control"
    control.parent.mkdir(parents=True)
    control.write_text(
        f"Package: sinesi\nVersion: {version}\nArchitecture: {arch}\n"
        "Section: education\nPriority: optional\n"
        "Maintainer: Sinesi contributors\n"
        "Depends: libc6 (>= 2.36), libgcc-s1, libxkbcommon0, libxkbcommon-x11-0, libwayland-client0, "
        "libx11-6, libxcursor1, libxi6, libxrandr2, libvulkan1, "
        "libegl1, desktop-file-utils, shared-mime-info\n"
        "Recommends: xdg-desktop-portal\n"
        "Description: Educational network simulator\n"
        " Design and simulate offline Ethernet and IPv4 networks.\n",
        encoding="utf-8",
    )
    control.chmod(0o644)
    for name in ("sns-desktop", "sinesi"):
        copy(bin_dir / name, root / "usr/bin" / name, executable=True)
    for source, relative in [
        (PACKAGING / "linux" / f"{APP_ID}.desktop", f"applications/{APP_ID}.desktop"),
        (PACKAGING / "linux" / f"{APP_ID}.xml", f"mime/packages/{APP_ID}.xml"),
        (ROOT / "assets/network/icon/router.svg", f"icons/hicolor/scalable/apps/{APP_ID}.svg"),
        (ROOT / "LICENSE", "doc/sinesi/copyright"),
        (ROOT / "assets/fonts/OFL.txt", "doc/sinesi/Noto-OFL.txt"),
        (ROOT / "assets/network/provenance.json", "doc/sinesi/network-asset-provenance.json"),
    ]:
        copy(source, root / "usr/share" / relative)
    run("dpkg-deb", "--root-owner-group", "--build", root, destination)


def macos(bin_dir, stage, destination, version, arch):
    app = stage / "Sinesi.app"
    contents = app / "Contents"
    executables = contents / "MacOS"
    for name in ("sns-desktop", "sinesi"):
        run("xcrun", "lipo", "-verify_arch", arch, bin_dir / name)
        copy(bin_dir / name, executables / name, executable=True)
    for source, name in [(ROOT / "LICENSE", "LICENSE"), (ROOT / "assets/fonts/OFL.txt", "Noto-OFL.txt")]:
        copy(source, contents / "Resources" / name)
    info = {
        "CFBundleDisplayName": "Sinesi",
        "CFBundleName": "Sinesi",
        "CFBundleIdentifier": APP_ID,
        "CFBundleExecutable": "sinesi-launcher",
        "CFBundlePackageType": "APPL",
        "CFBundleShortVersionString": version,
        "CFBundleVersion": version,
        "LSMinimumSystemVersion": "13.0",
        "LSUIElement": True,
        "NSHighResolutionCapable": True,
        "CFBundleDocumentTypes": [{
            "CFBundleTypeName": "Sinesi",
            "CFBundleTypeRole": "Editor",
            "LSHandlerRank": "Owner",
            "LSItemContentTypes": [PROJECT_UTI],
        }],
        "UTExportedTypeDeclarations": [{
            "UTTypeIdentifier": PROJECT_UTI,
            "UTTypeDescription": "Sinesi",
            "UTTypeConformsTo": ["public.json", "public.content"],
            "UTTypeTagSpecification": {
                "public.filename-extension": ["sinesi"],
                "public.mime-type": [PROJECT_MIME],
            },
        }],
    }
    with (contents / "Info.plist").open("wb") as plist:
        plistlib.dump(info, plist)
    run("swiftc", "-parse-as-library", "-O", "-target", f"{arch}-apple-macos13.0",
        PACKAGING / "macos/Launcher.swift", "-o", executables / "sinesi-launcher")
    for name in ("sns-desktop", "sinesi", "sinesi-launcher"):
        run("codesign", "--force", "--sign", "-", executables / name)
    run("codesign", "--force", "--sign", "-", app)
    run("codesign", "--verify", "--deep", "--strict", app)
    run("ditto", "-c", "-k", "--keepParent", app, destination)


def windows(bin_dir, stage, destination, version, _arch):
    for name in ("sns-desktop.exe", "sinesi.exe"):
        binary = bin_dir / name
        with binary.open("rb") as stream:
            dos = stream.read(64)
            if len(dos) == 64 and dos[:2] == b"MZ":
                stream.seek(int.from_bytes(dos[60:64], "little"))
                pe = stream.read(26)
            else:
                pe = b""
        machine = int.from_bytes(pe[4:6], "little")
        if pe[:4] != b"PE\0\0" or machine != 0x8664 or pe[24:26] != b"\x0b\x02":
            raise SystemExit(f"Expected a Windows x64 PE32+ binary: {binary}")
    compiler = shutil.which("ISCC")
    if not compiler:
        compiler = str(Path(os.environ.get("ProgramFiles(x86)", r"C:\Program Files (x86)"))
                       / "Inno Setup 6" / "ISCC.exe")
    run(compiler, f"/DAppVersion={version}", f"/DBinDir={bin_dir}",
        f"/DOutputDir={stage}", PACKAGING / "windows/Sinesi.iss")
    shutil.move(stage / f"Sinesi-{version}-windows-x64.exe", destination)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--bin-dir", type=Path, default=ROOT / "target/release")
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    host = platform.system()
    machine = platform.machine().lower()
    version = tomllib.loads((ROOT / "Cargo.toml").read_text())["workspace"]["package"]["version"]
    match host, machine:
        case "Linux", "x86_64" | "aarch64":
            arch = {"x86_64": "amd64", "aarch64": "arm64"}[machine]
            builder, name = linux, f"sinesi_{version}_{arch}.deb"
        case "Darwin", "x86_64" | "arm64":
            arch = machine
            builder, name = macos, f"Sinesi-{version}-macos-{arch}.zip"
        case "Windows", "amd64" | "x86_64":
            arch = "x64"
            builder, name = windows, f"Sinesi-{version}-windows-{arch}.exe"
        case _:
            parser.error(f"Unsupported packaging host: {host}/{machine}")
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=True)
    result = output / name
    if result.exists():
        parser.error(f"Package already exists: {result}")
    bin_dir = args.bin_dir.resolve()
    extension = ".exe" if host == "Windows" else ""
    for name in ("sns-desktop", "sinesi"):
        if not (bin_dir / (name + extension)).is_file():
            parser.error(f"Missing release binary: {bin_dir / (name + extension)}")
    with tempfile.TemporaryDirectory(prefix="sinesi-", dir=output) as temp:
        stage = Path(temp)
        artifact = stage / "package" / result.name
        artifact.parent.mkdir()
        builder(bin_dir, stage, artifact, version, arch)
        artifact.rename(result)
    print(result)


if __name__ == "__main__":
    main()
