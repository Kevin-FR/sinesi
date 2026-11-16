#!/bin/sh
# Default mode is safe without a logged-in graphical session.
set -eu
if [ "$#" -lt 2 ] || [ "$#" -gt 3 ] || { [ "$#" -eq 3 ] && [ "$3" != --association ] && [ "$3" != --keyboard ]; }; then
    echo "Usage: $0 package.zip new-work-directory [--association|--keyboard]" >&2
    exit 2
fi
mkdir "$2"
root=$(cd "$2" && pwd -P)
ditto -x -k "$1" "$root"
app="$root/Sinesi.app"
plist="$app/Contents/Info.plist"
codesign --verify --deep --strict "$app"
arch=$(uname -m)
for name in sinesi-launcher sns-desktop sinesi; do
    xcrun lipo -verify_arch "$arch" "$app/Contents/MacOS/$name"
done
python3 - "$plist" <<'PY'
import plistlib
import sys

with open(sys.argv[1], "rb") as stream:
    info = plistlib.load(stream)
assert info["CFBundleIdentifier"] == "fr.sinesi.Sinesi"
assert info["CFBundleExecutable"] == "sinesi-launcher"
assert [entry["LSItemContentTypes"] for entry in info["CFBundleDocumentTypes"]] == [["fr.sinesi.project"]]
exported, = info["UTExportedTypeDeclarations"]
assert exported["UTTypeIdentifier"] == "fr.sinesi.project"
assert exported["UTTypeTagSpecification"]["public.filename-extension"] == ["sinesi"]
PY
echo "OK — signed $arch bundle, launcher and .sinesi document declaration"
swiftc -target "$arch-apple-macos13.0" -parse-as-library \
    packaging/macos/SmokeAssociation.swift packaging/macos/SmokeKeyboard.swift -o "$root/smoke-association"

exercise="$root/exercise"
python3 packaging/smoke.py --desktop "$app/Contents/MacOS/sinesi-launcher" \
    --cli "$app/Contents/MacOS/sinesi" --exercise-dir "$exercise"
project="$exercise/- réseau de classe.sinesi"
shasum -a 256 "$project" > "$root/project.sha256"
case "${3-}" in
    --association) "$root/smoke-association" "$app" "$project" "$exercise" ;;
    --keyboard) "$root/smoke-association" "$app" "$project" "$exercise" --keyboard ;;
esac
shasum -a 256 -c "$root/project.sha256"
find "$exercise" -type f -exec shasum -a 256 {} + > "$root/exercise.sha256"
rm -r "$app"
shasum -a 256 -c "$root/exercise.sha256"
