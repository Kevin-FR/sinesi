import AppKit
import Darwin

func desktopProcess(_ arguments: [String]) throws -> Process {
    guard let directory = Bundle.main.executableURL?.deletingLastPathComponent() else {
        throw CocoaError(.fileNoSuchFile)
    }
    let process = Process()
    process.executableURL = directory.appendingPathComponent("sns-desktop")
    process.arguments = arguments
    return process
}

@MainActor
final class Launcher: NSObject, NSApplicationDelegate {
    private var desktopProcesses: [UUID: Process] = [:]
    private var receivedDocuments = false
    private var finishedLaunching = false

    func application(_ sender: NSApplication, openFiles filenames: [String]) {
        receivedDocuments = receivedDocuments || !filenames.isEmpty
        var success = true
        for filename in filenames {
            if !start(["--", filename]) { success = false }
        }
        sender.reply(toOpenOrPrint: success ? .success : .failure)
        stopIfIdle()
    }

    func applicationDidFinishLaunching(_ notification: Notification) {
        finishedLaunching = true
        if !receivedDocuments { _ = start([]) }
        stopIfIdle()
    }

    private func start(_ arguments: [String]) -> Bool {
        do {
            let id = UUID()
            let process = try desktopProcess(arguments)
            process.terminationHandler = { [weak self] _ in
                Task { @MainActor in
                    self?.desktopProcesses.removeValue(forKey: id)
                    self?.stopIfIdle()
                }
            }
            try process.run()
            desktopProcesses[id] = process
            return true
        } catch {
            NSLog("Sinesi: %@", error.localizedDescription)
            return false
        }
    }

    private func stopIfIdle() {
        if finishedLaunching && desktopProcesses.isEmpty {
            NSApplication.shared.terminate(nil)
        }
    }
}

@main
struct Main {
    @MainActor
    static func main() {
        let arguments = Array(CommandLine.arguments.dropFirst())
        if !arguments.isEmpty {
            // Explicit terminal invocation keeps its stdout, stderr and exit status.
            do {
                let process = try desktopProcess(arguments)
                try process.run()
                process.waitUntilExit()
                exit(process.terminationStatus)
            } catch {
                NSLog("Sinesi: %@", error.localizedDescription)
                exit(EXIT_FAILURE)
            }
        }
        let application = NSApplication.shared
        application.setActivationPolicy(.accessory)
        let delegate = Launcher()
        application.delegate = delegate
        withExtendedLifetime(delegate) {
            application.run()
        }
    }
}
