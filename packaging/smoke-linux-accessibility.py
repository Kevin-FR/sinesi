#!/usr/bin/env python3
"""Drive the installed X11 application by keyboard and read focus through AT-SPI."""
import json
import os
from pathlib import Path
import signal
import subprocess
import sys
import time

import pyatspi

ROOT = Path(__file__).resolve().parent.parent
DESKTOP = "/usr/bin/sns-desktop"
LANGUAGES = ("fr", "en", "es", "it", "pt", "de")


def run(*args, **kwargs):
    return subprocess.run(args, check=True, text=True, timeout=20, **kwargs)


def wait_for(description, probe):
    deadline = time.monotonic() + 30
    while time.monotonic() < deadline:
        value = probe()
        if value:
            return value
        time.sleep(0.05)
    raise RuntimeError(f"Timed out waiting for {description}")


def command_line(path):
    try:
        return path.read_bytes().split(b"\0")[:-1]
    except FileNotFoundError:
        return []


def exercise(directory, language, view, catalog, baseline, profile, desktop):
    case = f"{language}/{view}"
    directory.mkdir(parents=True)
    project = directory / "- réseau de classe.sinesi"
    project.write_bytes(baseline)
    original = json.loads(baseline)
    settings = directory / "settings.json"
    profile["language"] = language
    profile["display"].update(
        representation=view, scale_percent=150, window_width=1600, window_height=1050
    )
    settings.write_text(json.dumps(profile), encoding="utf-8")
    env = {
        **os.environ,
        "SINESI_SETTINGS_PATH": str(settings),
        "SINESI_SESSION_DIR": str(directory / "sessions"),
    }
    report = json.loads(
        run(DESKTOP, "--check-config", "--", str(project), env=env, capture_output=True).stdout
    )
    effective = report["configuration"]["effective"]
    assert effective["language"] == language, case
    assert effective["display"]["representation"] == view, case
    assert effective["display"]["scale_percent"] == 150, case

    run("gio", "open", str(project), env=env)
    expected = [os.fsencode(DESKTOP), b"--", os.fsencode(project)]
    process = wait_for(
        f"{case} associated desktop process",
        lambda: next(
            (p for p in Path("/proc").glob("[0-9]*/cmdline") if command_line(p) == expected),
            None,
        ),
    )
    pid = int(process.parent.name)

    try:
        run("xdotool", "search", "--sync", "--onlyvisible", "--pid", str(pid), capture_output=True)
        wait_for(
            f"{case} active window",
            lambda: subprocess.run(
                ["xdotool", "getactivewindow", "getwindowpid"],
                capture_output=True, text=True, timeout=5,
            ).stdout.strip() == str(pid),
        )
        app = wait_for(
            f"{case} AT-SPI application",
            lambda: next((a for a in desktop if a and a.get_process_id() == pid), None),
        )

        def press(key):
            run("xdotool", "key", key)

        def find(label):
            return pyatspi.findDescendant(app, lambda node: node.name == label)

        def activate(key, **values):
            label = catalog[key][language].format(**values)
            node = wait_for(f"{case} control {label!r}", lambda: find(label))
            for _ in range(160):
                if node.getState().contains(pyatspi.STATE_FOCUSED):
                    press("Return")
                    # Let the application commit activation before a later Tab moves focus.
                    time.sleep(0.15)
                    return
                press("Tab")
                time.sleep(0.07)
            raise RuntimeError(f"{case} did not expose keyboard focus on {label!r}")

        activate("add_notebook")
        new_device = catalog["notebook"][language] + f" {len(original['devices']) + 1}"
        device_label = catalog["canvas_device"][language].format(label=new_device)
        wait_for(f"{case} new laptop in the canvas", lambda: find(device_label))
        press("ctrl+s")
        wait_for(
            f"{case} project saved by Ctrl+S",
            lambda: len(json.loads(project.read_bytes())["devices"]) == len(original["devices"]) + 1,
        )
        saved = project.read_bytes()
        assert json.loads(saved)["links"] == original["links"], case
        press("F3")
        wait_for(f"{case} simulation mode", lambda: find(catalog["step"][language]))
        activate("canvas_device", label=original["devices"][0]["label"])
        activate("send_ping")
        activate("resume")
        activate("exchanges")
        wait_for(f"{case} trace page", lambda: find(catalog["traffic_detail"][language]))
        wait_for(f"{case} ICMP echo reply in the trace", lambda: find("icmp.echo.reply"))
        assert project.read_bytes() == saved, case

        press("alt+F4")
        wait_for(f"{case} clean window close", lambda: not command_line(process))
        print(f"OK — {case} at 150%: associated window, AT-SPI focus, Ctrl+S, ICMP reply")
    finally:
        if command_line(process) == expected:
            os.kill(pid, signal.SIGTERM)


def main():
    directory = Path(sys.argv[1]).resolve()
    catalog = json.loads((ROOT / "crates/sns-i18n/locales/messages.json").read_bytes())
    baseline = (ROOT / "fixtures/valid/lan.json").read_bytes()
    profile = json.loads((ROOT / "fixtures/valid/preferences-v2.json").read_bytes())
    run(
        "gdbus", "call", "--session", "--dest", "org.a11y.Bus", "--object-path", "/org/a11y/bus",
        "--method", "org.freedesktop.DBus.Properties.Set", "org.a11y.Status",
        "ScreenReaderEnabled", "<true>",
    )
    desktop = pyatspi.Registry.getDesktop(0)
    for language in LANGUAGES:
        for view in ("icons", "diagram"):
            exercise(directory / language / view, language, view, catalog, baseline, profile, desktop)


if __name__ == "__main__":
    main()
