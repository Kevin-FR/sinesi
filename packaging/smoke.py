#!/usr/bin/env python3
"""Exercise a packaged desktop launcher and its companion CLI in an isolated directory."""
import argparse
import json
import os
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parent.parent


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--desktop", type=Path, required=True)
    parser.add_argument("--cli", type=Path, required=True)
    parser.add_argument("--exercise-dir", type=Path, required=True)
    args = parser.parse_args()
    workspace = args.exercise_dir.resolve()
    workspace.mkdir(parents=True)
    desktop, cli = args.desktop.resolve(strict=True), args.cli.resolve(strict=True)
    environment = {
        **os.environ,
        "SINESI_SETTINGS_PATH": str(workspace / "settings.json"),
        "SINESI_SESSION_DIR": str(workspace / "sessions"),
    }

    def run(*command):
        result = subprocess.run(command, cwd=workspace, env=environment,
                                capture_output=True, encoding="utf-8", timeout=60)
        if result.returncode:
            raise SystemExit(f"{command[0]} failed ({result.returncode}): {result.stderr.strip()}")
        return result.stdout

    project = workspace / "- réseau de classe.sinesi"
    original = ROOT / "fixtures/legacy/lan.fls"
    run(cli, "import-filius", original, "--output", project, "--report", "import.json")
    expected = json.loads(project.read_bytes())
    source = project.read_bytes()
    expected_counts = {key: len(expected[key]) for key in ("devices", "links")}
    for language in ("fr", "en", "es", "it", "pt", "de"):
        for view in ("icons", "diagram"):
            report = json.loads(run(desktop, "--check-config", "--language", language,
                                    "--view", view, "--", project.name))
            assert report["project"] == {
                "schema_version": expected["schema_version"], **expected_counts
            }
            effective = report["configuration"]["effective"]
            assert effective["language"] == language
            assert effective["display"]["representation"] == view
    assert project.read_bytes() == source
    scenario = ROOT / "fixtures/valid/network-scenario.json"
    run(cli, "run", scenario, "--until-us", "10000000", "--recording", "recording.json",
        "--checkpoint", "checkpoint.json", "--trace", "original.jsonl")
    run(cli, "replay", "recording.json", "--trace", "replayed.jsonl")
    original_trace = (workspace / "original.jsonl").read_bytes()
    assert original_trace == (workspace / "replayed.jsonl").read_bytes()
    assert any(json.loads(line)["type"] == "icmp.echo.reply" for line in original_trace.splitlines())
    for view in ("icons", "diagram"):
        target = workspace / f"network-{view}.png"
        result = json.loads(run(cli, "render-png", project, "--output", target, "--view", view))
        assert result["view"] == view
        assert target.read_bytes().startswith(b"\x89PNG\r\n\x1a\n")
    print(f"OK — 12 startup combinations, portable save, ping, checkpoint, replay and PNG in {workspace}")


if __name__ == "__main__":
    main()
