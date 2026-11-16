import AppKit
import ApplicationServices
import Carbon

enum KeyboardError: Error {
    case accessibilityPermissionRequired
    case invalidFixture
    case incorrectSettings(String)
    case timedOut(String)
    case lostForeground
    case eventCreationFailed
    case linksChanged
    case simulationChangedProject
}

struct KeyboardProject: Decodable {
    struct Device: Decodable { let label: String }
    struct Link: Decodable, Equatable {
        let id: String
        let endpoints: [String]
        let latency_us: String
        let jitter_us: String
    }
    let devices: [Device]
    let links: [Link]
}

struct KeyboardCase {
    let directory: URL
    let project: URL
    let language: String
    let view: String
    let labels: [String: String]

    var description: String { "\(language)/\(view)" }
    var environment: [String: String] {
        [
            "SINESI_SETTINGS_PATH": directory.appendingPathComponent("settings.json").path,
            "SINESI_SESSION_DIR": directory.appendingPathComponent("sessions").path,
        ]
    }

    static func prepare(in exercise: URL, filename: String) throws -> [KeyboardCase] {
        let catalog = try JSONDecoder().decode(
            [String: [String: String]].self,
            from: Data(contentsOf: URL(fileURLWithPath: "crates/sns-i18n/locales/messages.json"))
        )
        let template = try Data(contentsOf: URL(fileURLWithPath: "fixtures/valid/preferences-v2.json"))
        let baseline = try Data(contentsOf: URL(fileURLWithPath: "fixtures/valid/lan.json"))
        let required = ["add_notebook", "notebook", "canvas_device", "step", "send_ping", "resume", "exchanges"]
        var cases: [KeyboardCase] = []
        for language in ["fr", "en", "es", "it", "pt", "de"] {
            var labels: [String: String] = [:]
            for key in required {
                guard let label = catalog[key]?[language] else { throw KeyboardError.invalidFixture }
                labels[key] = label
            }
            for view in ["icons", "diagram"] {
                guard var profile = try JSONSerialization.jsonObject(with: template) as? [String: Any],
                      var display = profile["display"] as? [String: Any] else {
                    throw KeyboardError.invalidFixture
                }
                let directory = exercise.appendingPathComponent("keyboard/\(language)/\(view)")
                try FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)
                let project = directory.appendingPathComponent(filename)
                try baseline.write(to: project, options: .withoutOverwriting)
                profile["language"] = language
                display["representation"] = view
                display["scale_percent"] = 150
                display["window_width"] = 1600
                display["window_height"] = 1050
                profile["display"] = display
                let settings = try JSONSerialization.data(withJSONObject: profile)
                try settings.write(to: directory.appendingPathComponent("settings.json"), options: .withoutOverwriting)
                cases.append(KeyboardCase(directory: directory, project: project, language: language, view: view, labels: labels))
            }
        }
        return cases
    }

    func verifySettings(executable: URL) throws {
        let process = Process()
        process.executableURL = executable
        process.arguments = ["--check-config", "--", project.path]
        process.environment = ProcessInfo.processInfo.environment.merging(environment) { _, new in new }
        let pipe = Pipe()
        process.standardOutput = pipe
        try process.run()
        let data = pipe.fileHandleForReading.readDataToEndOfFile()
        process.waitUntilExit()
        guard process.terminationStatus == 0,
              let report = try JSONSerialization.jsonObject(with: data) as? [String: Any],
              let configuration = report["configuration"] as? [String: Any],
              let effective = configuration["effective"] as? [String: Any],
              let display = effective["display"] as? [String: Any],
              effective["language"] as? String == language,
              display["representation"] as? String == view,
              display["scale_percent"] as? Int == 150 else {
            throw KeyboardError.incorrectSettings(description)
        }
    }
}

@MainActor
func exerciseKeyboard(_ test: KeyboardCase, pid: pid_t) async throws {
    let application = AXUIElementCreateApplication(pid)
    let system = AXUIElementCreateSystemWide()
    AXUIElementSetMessagingTimeout(system, 1)
    defer { AXUIElementSetMessagingTimeout(system, 0) }
    let decoder = JSONDecoder()
    let original = try decoder.decode(KeyboardProject.self, from: Data(contentsOf: test.project))
    guard let source = original.devices.first,
          let add = test.labels["add_notebook"], let notebook = test.labels["notebook"],
          let canvas = test.labels["canvas_device"], let step = test.labels["step"],
          let ping = test.labels["send_ping"], let resume = test.labels["resume"],
          let exchanges = test.labels["exchanges"] else { throw KeyboardError.invalidFixture }

    func attribute<T>(_ node: AXUIElement, _ name: String) -> T? {
        var value: CFTypeRef?
        guard AXUIElementCopyAttributeValue(node, name as CFString, &value) == .success else { return nil }
        return value as? T
    }

    func matches(_ node: AXUIElement, _ label: String) -> Bool {
        for name in [kAXTitleAttribute, kAXDescriptionAttribute, kAXValueAttribute] {
            if let value: String = attribute(node, name), value == label { return true }
        }
        return false
    }

    func contains(_ label: String) -> Bool {
        let clock = ContinuousClock()
        let deadline = clock.now.advanced(by: .seconds(2))
        var pending: [AXUIElement] = attribute(application, kAXWindowsAttribute) ?? []
        for _ in 0..<4000 {
            guard clock.now < deadline, let node = pending.popLast() else { break }
            if matches(node, label) { return true }
            pending.append(contentsOf: attribute(node, kAXChildrenAttribute) ?? [])
        }
        return false
    }

    func waitFor(_ label: String, _ predicate: () throws -> Bool) async throws {
        let clock = ContinuousClock()
        let deadline = clock.now.advanced(by: .seconds(20))
        repeat {
            if try predicate() { return }
            try await Task.sleep(for: .milliseconds(75))
        } while clock.now < deadline
        throw KeyboardError.timedOut("\(test.description) \(label)")
    }

    func press(_ code: Int, flags: CGEventFlags = []) throws {
        guard NSWorkspace.shared.frontmostApplication?.processIdentifier == pid else {
            throw KeyboardError.lostForeground
        }
        for down in [true, false] {
            guard let event = CGEvent(keyboardEventSource: nil, virtualKey: CGKeyCode(code), keyDown: down) else {
                throw KeyboardError.eventCreationFailed
            }
            event.flags = flags
            event.postToPid(pid)
        }
    }

    func activate(_ label: String) async throws {
        try await waitFor(label) { contains(label) }
        for _ in 0..<160 {
            if let focused: AXUIElement = attribute(system, kAXFocusedUIElementAttribute) {
                var owner: pid_t = 0
                if AXUIElementGetPid(focused, &owner) == .success, owner == pid, matches(focused, label) {
                    try press(kVK_Return)
                    try await Task.sleep(for: .milliseconds(150))
                    return
                }
            }
            try press(kVK_Tab)
            try await Task.sleep(for: .milliseconds(75))
        }
        throw KeyboardError.timedOut("\(test.description) focus on \(label)")
    }

    _ = NSRunningApplication(processIdentifier: pid)?.activate()
    try await waitFor("foreground window") { NSWorkspace.shared.frontmostApplication?.processIdentifier == pid }
    try await activate(add)
    let newCount = original.devices.count + 1
    let newLabel = canvas.replacingOccurrences(of: "{label}", with: "\(notebook) \(newCount)")
    try await waitFor(newLabel) { contains(newLabel) }
    try press(kVK_ANSI_S, flags: .maskCommand)
    try await waitFor("Command+S") {
        try decoder.decode(KeyboardProject.self, from: Data(contentsOf: test.project)).devices.count == newCount
    }
    let saved = try Data(contentsOf: test.project)
    guard try decoder.decode(KeyboardProject.self, from: saved).links == original.links else {
        throw KeyboardError.linksChanged
    }
    try press(kVK_F3)
    try await waitFor(step) { contains(step) }
    try await activate(canvas.replacingOccurrences(of: "{label}", with: source.label))
    try await activate(ping)
    try await activate(resume)
    try await activate(exchanges)
    try await waitFor("ICMP reply") { contains("icmp.echo.reply") }
    guard try Data(contentsOf: test.project) == saved else { throw KeyboardError.simulationChangedProject }
    print("OK — \(test.description) at 150%: associated window, AX focus, Command+S, ICMP reply")
}
