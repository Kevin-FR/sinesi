import AppKit
import ApplicationServices
import CoreGraphics
import Darwin

private enum SmokeError: Error {
    case usage
    case unexpectedApplication
    case unexpectedChildren([String])
    case windowMissing
    case launcherDidNotExit
}

private func output(_ executable: String, _ arguments: [String]) throws -> String {
    let process = Process()
    process.executableURL = URL(fileURLWithPath: executable)
    process.arguments = arguments
    let pipe = Pipe()
    process.standardOutput = pipe
    try process.run()
    let bytes = pipe.fileHandleForReading.readDataToEndOfFile()
    process.waitUntilExit()
    return String(decoding: bytes, as: UTF8.self)
}

private func desktopChildren(_ parent: pid_t) throws -> [(pid_t, String)] {
    let ids = try output("/usr/bin/pgrep", ["-P", String(parent), "-x", "sns-desktop"])
    return try ids.split(whereSeparator: \.isWhitespace).compactMap { field in
        guard let pid = pid_t(field) else { return nil }
        let command = try output("/bin/ps", ["-ww", "-o", "command=", "-p", String(pid)])
        return (pid, command.trimmingCharacters(in: .newlines))
    }
}

private func hasWindow(_ pid: pid_t) -> Bool {
    let windows = CGWindowListCopyWindowInfo(.excludeDesktopElements, kCGNullWindowID)
        as? [[String: Any]] ?? []
    return windows.contains {
        ($0[kCGWindowOwnerPID as String] as? Int32) == pid &&
            ($0[kCGWindowLayer as String] as? Int) == 0
    }
}

@main
@MainActor
struct SmokeAssociation {
    static func main() async {
        do {
            try await check()
        } catch {
            FileHandle.standardError.write(Data("macOS association check failed: \(error)\n".utf8))
            exit(EXIT_FAILURE)
        }
    }

    private static func check() async throws {
        var arguments = Array(CommandLine.arguments.dropFirst())
        let keyboard = arguments.last == "--keyboard"
        if keyboard { arguments.removeLast() }
        guard arguments.count == 3 else { throw SmokeError.usage }
        let paths = arguments.map {
            URL(fileURLWithPath: $0).resolvingSymlinksInPath()
        }
        let bundle = paths[0]
        let project = paths[1]
        let exercise = paths[2]
        if keyboard {
            guard AXIsProcessTrusted(), CGPreflightPostEventAccess() else {
                throw KeyboardError.accessibilityPermissionRequired
            }
            for test in try KeyboardCase.prepare(in: exercise, filename: project.lastPathComponent) {
                try test.verifySettings(executable: bundle.appendingPathComponent("Contents/MacOS/sns-desktop"))
                try await checkAssociation(bundle, project: test.project, exercise: test.directory, keyboard: test)
            }
        } else {
            try await checkAssociation(bundle, project: project, exercise: exercise)
        }
    }

    private static func checkAssociation(
        _ bundle: URL, project: URL, exercise: URL, keyboard: KeyboardCase? = nil
    ) async throws {
        let config = NSWorkspace.OpenConfiguration()
        config.createsNewApplicationInstance = true
        config.activates = false
        config.addsToRecentItems = false
        config.environment = [
            "SINESI_SETTINGS_PATH": exercise.appendingPathComponent("settings.json").path,
            "SINESI_SESSION_DIR": exercise.appendingPathComponent("sessions").path,
        ]
        let launcher = try await NSWorkspace.shared.open(
            [project], withApplicationAt: bundle, configuration: config
        )
        guard launcher.bundleURL?.resolvingSymlinksInPath().path == bundle.path else {
            throw SmokeError.unexpectedApplication
        }
        let executable = bundle.appendingPathComponent("Contents/MacOS/sns-desktop").path
        let expected = "\(executable) -- \(project.path)"
        defer {
            for (pid, command) in (try? desktopChildren(launcher.processIdentifier)) ?? []
                where command == executable || command.hasPrefix(executable + " ") {
                kill(pid, SIGTERM)
            }
            if !launcher.isTerminated { _ = launcher.terminate() }
        }

        for _ in 0..<300 {
            let children = try desktopChildren(launcher.processIdentifier)
            if children.count > 1 || (children.count == 1 && children[0].1 != expected) {
                throw SmokeError.unexpectedChildren(children.map(\.1))
            }
            if let (pid, _) = children.first, hasWindow(pid) {
                print("OK — LaunchServices delivered the Unicode project to the bundled desktop window")
                if let keyboard { try await exerciseKeyboard(keyboard, pid: pid) }
                kill(pid, SIGTERM)
                for _ in 0..<50 {
                    if launcher.isTerminated { return }
                    try await Task.sleep(for: .milliseconds(100))
                }
                throw SmokeError.launcherDidNotExit
            }
            try await Task.sleep(for: .milliseconds(100))
        }
        throw SmokeError.windowMissing
    }
}
