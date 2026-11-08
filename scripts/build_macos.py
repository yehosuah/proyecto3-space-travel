#!/usr/bin/env python3
"""Build a relocatable macOS preview using the checked-in dependency lockfile."""
from pathlib import Path
import plistlib
import shutil
import subprocess
import sys

root = Path(__file__).resolve().parents[1]
if sys.platform != "darwin":
    raise SystemExit("Este empaquetador crea la aplicación para macOS.")
subprocess.run(["cargo", "build", "--release", "--locked"], cwd=root, check=True)
contents = root / "COSMOS GP.app" / "Contents"
(contents / "MacOS").mkdir(parents=True, exist_ok=True)
(contents / "Resources").mkdir(parents=True, exist_ok=True)
replacement=contents / "MacOS/.cosmos-gp.new"
shutil.copy2(root / "target/release/cosmos-gp",replacement)
replacement.replace(contents / "MacOS/cosmos-gp")
shutil.copytree(root / "assets", contents / "Resources/assets", dirs_exist_ok=True)
with (contents / "Info.plist").open("wb") as f:
    plistlib.dump({
        "CFBundleExecutable": "cosmos-gp",
        "CFBundleIdentifier": "edu.uvg.yehosua.cosmosgp",
        "CFBundleName": "COSMOS GP",
        "CFBundleDisplayName": "COSMOS / GP",
        "CFBundlePackageType": "APPL",
        "CFBundleShortVersionString": "0.1.0",
        "CFBundleVersion": "1",
        "NSHighResolutionCapable": True,
        "NSHumanReadableCopyright": "Proyecto académico de Yehosua F. Hercules",
    }, f)
print(contents.parent)
