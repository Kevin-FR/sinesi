#!/bin/sh
set -eu
cx='/Applications/CrossOver.app/Contents/SharedSupport/CrossOver/bin'
export CX_BOTTLE_PATH=/tmp/sns-j7-win64/bottles
export WINEDLLOVERRIDES='winemenubuilder.exe=d;mscoree,mshtml='
bottle="$CX_BOTTLE_PATH/SinesiJ7QA"
work=/tmp/sns-j7-win64
installer=Sinesi-0.1.0-windows-x64.exe
wine() { "$cx/wine" --bottle SinesiJ7QA --cx-app "$@"; }
reg() { wine 'C:\windows\system32\reg.exe' "$@"; }
test ! -e "$bottle/drive_c/SinesiQA"
cp "$work/cx-installer/$installer" "$bottle/drive_c/"
wine "C:\\$installer" /VERYSILENT /SUPPRESSMSGBOXES /NORESTART '/DIR=C:\SinesiQA'
test -f "$bottle/drive_c/SinesiQA/sinesi.exe"
test -f "$bottle/drive_c/SinesiQA/sns-desktop.exe"
cmp "$work/sinesi.exe" "$bottle/drive_c/SinesiQA/sinesi.exe"
cmp "$work/sns-desktop.exe" "$bottle/drive_c/SinesiQA/sns-desktop.exe"
reg query 'HKCU\Software\Classes\Sinesi.Project\shell\open\command' /ve
reg query 'HKCU\Software\Classes\.sinesi\OpenWithProgids' /v Sinesi.Project
mkdir -p "$work/cx-shims"
cat >"$work/cx-shims/sinesi.exe" <<'PY'
#!/usr/bin/env python3
import os
from pathlib import Path
import sys

def wine_path(arg):
    return "Z:" + arg.replace("/", "\\") if arg.startswith("/") else arg

for name in ("SINESI_SETTINGS_PATH", "SINESI_SESSION_DIR"):
    os.environ[name] = wine_path(os.environ[name])
name = Path(sys.argv[0]).name
wine = "/Applications/CrossOver.app/Contents/SharedSupport/CrossOver/bin/wine"
os.execv(wine, [wine, "--bottle", "SinesiJ7QA", "--no-convert", "--cx-app",
                "C:\\SinesiQA\\" + name, "--", *map(wine_path, sys.argv[1:])])
PY
cp "$work/cx-shims/sinesi.exe" "$work/cx-shims/sns-desktop.exe"
chmod +x "$work/cx-shims/sinesi.exe" "$work/cx-shims/sns-desktop.exe"
python3 packaging/smoke.py --desktop "$work/cx-shims/sns-desktop.exe" --cli "$work/cx-shims/sinesi.exe" --exercise-dir "$work/cx-exercise"
shasum -a 256 "$work"/cx-exercise/*.sinesi >"$work/cx-exercise/project.sha256"
wine 'C:\SinesiQA\unins000.exe' /VERYSILENT /SUPPRESSMSGBOXES /NORESTART
test ! -e "$bottle/drive_c/SinesiQA/sinesi.exe"
if reg query 'HKCU\Software\Classes\Sinesi.Project' >/dev/null 2>&1; then exit 1; fi
if reg query 'HKCU\Software\Classes\.sinesi\OpenWithProgids' /v Sinesi.Project >/dev/null 2>&1; then exit 1; fi
shasum -a 256 -c "$work/cx-exercise/project.sha256"
