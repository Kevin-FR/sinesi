#!/bin/sh
# Run only in a fresh container. The exercise directory is outside the package.
set -eu
test -f /.dockerenv
if dpkg-query -W sinesi >/dev/null 2>&1; then
    echo 'Sinesi is already registered with dpkg' >&2
    exit 1
fi
package=$(realpath "$1")
exercise=$(realpath -m "$2")
apt-get install -y --no-install-recommends "$package"
desktop=/usr/share/applications/fr.sinesi.Sinesi.desktop
desktop-file-validate "$desktop"
python3 packaging/smoke.py --desktop /usr/bin/sns-desktop --cli /usr/bin/sinesi --exercise-dir "$exercise"
project="$exercise/- réseau de classe.sinesi"
mime=application/vnd.sinesi.project+json
gio info -a standard::content-type "$project" | grep -F "standard::content-type: $mime"
export XDG_CONFIG_HOME="$exercise/desktop-config"
export XDG_RUNTIME_DIR="$exercise/desktop-runtime"
export SINESI_SETTINGS_PATH="$exercise/settings.json"
export SINESI_SESSION_DIR="$exercise/sessions"
mkdir -m 700 "$XDG_CONFIG_HOME" "$XDG_RUNTIME_DIR"
gio mime "$mime" | tee "$exercise/mime.txt"
grep -F 'fr.sinesi.Sinesi.desktop' "$exercise/mime.txt"
gio mime "$mime" fr.sinesi.Sinesi.desktop
dbus-run-session -- xvfb-run -a -s "-screen 0 1680x1100x24" sh -c '
    openbox > "$1/openbox.log" 2>&1 &
    exec python3 -u packaging/smoke-linux-accessibility.py "$1/interactive"
' sh "$exercise"
sha256sum "$project" > "$exercise/project.sha256"
find "$exercise/interactive" -type f -exec sha256sum {} + >> "$exercise/project.sha256"
dpkg --remove sinesi
test ! -e "$desktop"
test ! -e /usr/bin/sns-desktop
test ! -e /usr/share/mime/packages/fr.sinesi.Sinesi.xml
sha256sum -c "$exercise/project.sha256"
