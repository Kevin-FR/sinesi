//! R-UX-001 to R-UX-004: one workbench, Student/Expert levels and three named workspaces.
use egui_kittest::{
    Harness,
    kittest::{NodeT as _, Queryable as _},
};
use sns_i18n::Language;
use sns_settings::display::Level;
use sns_ui_egui::{Mode, Workbench};

fn harness(app: Workbench) -> Harness<'static, Workbench> {
    Harness::builder()
        .with_max_steps(60)
        .with_size((1400., 900.))
        .build_ui_state(|ui, state| state.show(ui), app)
}
fn settle(h: &mut Harness<'_, Workbench>) {
    h.step();
    h.state_mut().wait_for_worker().unwrap();
    h.run();
}
const EXPERT_COMMANDS: [&str; 3] = ["import_filius", "runtime_menu", "export_network"];

#[test]
fn student_level_is_default_and_expert_reveals_the_same_commands_without_touching_the_project() {
    for l in Language::ALL {
        let mut app = Workbench::new().unwrap();
        app.choose_language(l);
        assert_eq!(app.level(), Level::Student);
        let mut h = harness(app);
        h.run();
        let project = h.state().project().clone();
        for key in EXPERT_COMMANDS {
            assert!(
                h.query_by_label(l.text(key)).is_none(),
                "{key} visible to students"
            );
        }
        h.get_by_label(l.text("level_expert")).click();
        h.run();
        assert_eq!(h.state().level(), Level::Expert);
        for key in EXPERT_COMMANDS {
            h.get_by_label(l.text(key));
        }
        assert_eq!(h.state().project(), &project);
        assert_eq!(h.state().mode, Mode::Design);
        h.get_by_label(l.text("level_student")).click();
        h.run();
        assert!(h.query_by_label(l.text("runtime_menu")).is_none());
    }
}

#[test]
fn chosen_level_is_read_back_and_a_profile_without_level_means_student() {
    let dir = tempfile::tempdir().unwrap();
    let prefs = dir.path().join("settings.json");
    std::fs::write(
        &prefs,
        include_bytes!("../../../fixtures/valid/preferences-v2.json"),
    )
    .unwrap();
    let mut app = Workbench::with_preferences(Some(prefs.clone())).unwrap();
    assert_eq!(app.level(), Level::Student);
    app.choose_level(Level::Expert);
    app.wait_for_preferences().unwrap();
    assert_eq!(
        sns_settings::load(&prefs).unwrap().display.level,
        Level::Expert
    );
    let reopened = Workbench::with_preferences(Some(prefs)).unwrap();
    assert_eq!(reopened.level(), Level::Expert);
}

#[test]
fn three_workspaces_follow_clicks_and_function_keys_with_an_instruction_in_six_languages() {
    #[cfg(feature = "ui-snapshots")]
    let mut snapshots = egui_kittest::SnapshotResults::new();
    for l in Language::ALL {
        for diagram in [false, true] {
            let mut app = Workbench::new().unwrap();
            app.choose_language(l);
            let mut h = harness(app);
            h.run();
            h.get_by_label(l.text(if diagram {
                "view_diagram"
            } else {
                "view_icons"
            }))
            .click();
            h.run();
            for (mode, key, shortcut, hint, name) in [
                (
                    Mode::Simulation,
                    "mode_test",
                    egui::Key::F3,
                    "hint_test",
                    "test",
                ),
                (
                    Mode::Documentation,
                    "mode_annotate",
                    egui::Key::F4,
                    "hint_annotate",
                    "annotate",
                ),
                (
                    Mode::Design,
                    "mode_build",
                    egui::Key::F2,
                    "hint_build",
                    "build",
                ),
            ] {
                let label = format!("{} · {:?}", l.text(key), shortcut);
                h.get_by_label(&label).click();
                settle(&mut h);
                assert_eq!(h.state().mode, mode);
                h.get_by_label(l.text(hint));
                #[cfg(feature = "ui-snapshots")]
                {
                    let view = if diagram { "diagram" } else { "icons" };
                    h.snapshot(format!("j8-student-{name}-{view}-{}", l.code()));
                    snapshots.extend_harness(&mut h);
                }
                #[cfg(not(feature = "ui-snapshots"))]
                let _ = name;
            }
            for (mode, key) in [
                (Mode::Simulation, egui::Key::F3),
                (Mode::Documentation, egui::Key::F4),
                (Mode::Design, egui::Key::F2),
            ] {
                h.key_press(key);
                settle(&mut h);
                assert_eq!(h.state().mode, mode);
            }
        }
    }
}

#[test]
fn illustrated_palette_and_guided_cable_connect_a_new_computer_in_six_languages() {
    #[cfg(feature = "ui-snapshots")]
    let mut snapshots = egui_kittest::SnapshotResults::new();
    for l in Language::ALL {
        for diagram in [false, true] {
            let mut app = Workbench::new().unwrap();
            app.choose_language(l);
            let mut h = harness(app);
            h.run();
            h.get_by_label(l.text(if diagram {
                "view_diagram"
            } else {
                "view_icons"
            }))
            .click();
            h.run();
            let devices = h.state().project().devices.len();
            let links = h.state().project().links.len();
            h.get_by_label(l.text("add_workstation")).click();
            h.run();
            assert_eq!(h.state().project().devices.len(), devices + 1);
            let added = h.state().project().devices.last().unwrap().clone();
            let switch = h
                .state()
                .project()
                .devices
                .iter()
                .find(|d| d.kind == sns_model::DeviceKind::Switch)
                .unwrap()
                .clone();
            h.get_by_label(&format!("{} · C", l.text("cable_tool")))
                .click();
            h.run();
            h.get_by_label(l.text("cable_step_first"));
            let device = |name: &str| l.format("canvas_device", &[("label", name)]).unwrap();
            h.get_by_label(&device(&added.label)).click();
            h.run();
            h.get_by_label(
                &l.format("cable_step_second", &[("device", &added.label)])
                    .unwrap(),
            );
            #[cfg(feature = "ui-snapshots")]
            {
                let view = if diagram { "diagram" } else { "icons" };
                h.snapshot(format!("j8-cable-{view}-{}", l.code()));
                snapshots.extend_harness(&mut h);
            }
            h.get_by_label(&device(&switch.label)).click();
            h.run();
            assert_eq!(h.state().project().links.len(), links + 1);
            h.get_by_label(
                &l.format("cable_done", &[("a", &added.label), ("b", &switch.label)])
                    .unwrap(),
            );
            assert_eq!(h.state().mode, Mode::Design);
        }
    }
}

fn routed(gateway: bool) -> sns_model::Project {
    let mut project =
        sns_project::parse_project(include_bytes!("../../../fixtures/valid/routed.json")).unwrap();
    if !gateway {
        project.devices[0].network.default_gateway = None;
    }
    project
}

/// Select A, ping B through two routers, and step until the trace holds `kind`.
fn ping_from_a(h: &mut Harness<'_, Workbench>, l: Language, kind: &str) -> usize {
    h.get_by_label(&l.format("canvas_device", &[("label", "A")]).unwrap())
        .click();
    h.run();
    h.get_by_label(&format!("{} · F3", l.text("mode_test")))
        .click();
    settle(h);
    h.get_by_label(l.text("ping_target"))
        .set_text_value_accesskit("192.168.30.2");
    h.run();
    h.get_by_label(l.text("send_ping")).click();
    settle(h);
    let mut most_in_flight = 0;
    for _ in 0..80 {
        if h.state().trace().iter().any(|e| e.kind == kind) {
            break;
        }
        h.get_by_label(l.text("step")).click();
        h.step();
        h.state_mut().wait_for_worker().unwrap();
        h.step();
        // Flights last a fraction of a second; measure before the harness settles.
        let in_flight = h.state().packets_in_flight();
        #[cfg(feature = "ui-snapshots")]
        if in_flight > 0 && most_in_flight == 0 && kind == "icmp.echo.reply" {
            // One harness step (a quarter second) puts the first packet mid-cable.
            h.step();
            let view = if h.state().display_preferences().representation
                == sns_settings::display::Representation::Diagram
            {
                "diagram"
            } else {
                "icons"
            };
            h.snapshot(format!("j8-packet-{view}-{}", l.code()));
        }
        most_in_flight = most_in_flight.max(in_flight);
        h.run();
    }
    assert!(h.state().trace().iter().any(|e| e.kind == kind), "{kind}");
    most_in_flight
}

#[test]
fn packet_journey_is_animated_and_told_from_the_trace_including_a_missing_gateway() {
    #[cfg(feature = "ui-snapshots")]
    let mut snapshots = egui_kittest::SnapshotResults::new();
    for l in Language::ALL {
        for diagram in [false, true] {
            #[cfg(feature = "ui-snapshots")]
            let view = if diagram { "diagram" } else { "icons" };
            let mut app = Workbench::from_project(routed(true)).unwrap();
            app.choose_language(l);
            let mut h = harness(app);
            h.run();
            h.get_by_label(l.text(if diagram {
                "view_diagram"
            } else {
                "view_icons"
            }))
            .click();
            h.run();
            assert!(h.query_by_label(l.text("story_title")).is_none());
            let flights = ping_from_a(&mut h, l, "icmp.echo.reply");
            h.get_by_label(l.text("story_title"));
            assert!(flights > 0, "no packet was animated");
            let trace = h.state().trace().to_vec();
            h.run();
            let rtt = trace
                .iter()
                .find(|e| e.kind == "icmp.echo.reply")
                .and_then(|e| e.payload.get("rtt_us"))
                .and_then(|v| v.as_str())
                .unwrap()
                .parse::<u64>()
                .unwrap();
            for (key, params) in [
                (
                    "story_ping_sent",
                    vec![("device", "A"), ("target", "192.168.30.2")],
                ),
                (
                    "story_forwarded",
                    vec![("device", "R1"), ("next_hop", "192.168.20.2")],
                ),
                (
                    "story_echo_received",
                    vec![("device", "B"), ("source", "192.168.10.2")],
                ),
            ] {
                h.get_by_label(&l.format(key, &params).unwrap());
            }
            let ms = rtt.div_ceil(1000).to_string();
            h.get_by_label(
                &l.format(
                    "story_echo_reply",
                    &[("device", "A"), ("source", "192.168.30.2"), ("ms", &ms)],
                )
                .unwrap(),
            );
            // The story never adds facts: the trace is unchanged by drawing it.
            assert_eq!(h.state().trace(), trace.as_slice());
            #[cfg(feature = "ui-snapshots")]
            {
                h.snapshot(format!("j8-story-success-{view}-{}", l.code()));
                snapshots.extend_harness(&mut h);
            }

            let mut app = Workbench::from_project(routed(false)).unwrap();
            app.choose_language(l);
            let mut h = harness(app);
            h.run();
            h.get_by_label(l.text(if diagram {
                "view_diagram"
            } else {
                "view_icons"
            }))
            .click();
            h.run();
            ping_from_a(&mut h, l, "icmp.error");
            h.run();
            h.get_by_label(&l.format("story_no_route", &[("device", "A")]).unwrap());
            #[cfg(feature = "ui-snapshots")]
            {
                h.snapshot(format!("j8-story-no-gateway-{view}-{}", l.code()));
                snapshots.extend_harness(&mut h);
            }
        }
    }
}

fn terminal(h: &mut Harness<'_, Workbench>, l: Language, command: &str) {
    h.get_by_label(l.text("terminal_input")).focus();
    h.run();
    h.get_by_label(l.text("terminal_input")).type_text(command);
    h.run();
    h.key_press(egui::Key::Enter);
    settle(h);
}

#[test]
fn terminal_results_read_as_sentences_for_students_and_stay_structured_for_experts() {
    #[cfg(feature = "ui-snapshots")]
    let mut snapshots = egui_kittest::SnapshotResults::new();
    for l in Language::ALL {
        let mut app = Workbench::from_project(
            sns_project::parse_project(include_bytes!("../../../fixtures/valid/hosts.json"))
                .unwrap(),
        )
        .unwrap();
        app.choose_language(l);
        let mut h = Harness::builder()
            .with_max_steps(20)
            .with_size((1440., 1000.))
            .build_ui_state(|ui, state| state.show(ui), app);
        h.run();
        h.get_by_label(&format!("{} · F3", l.text("mode_test")))
            .click();
        settle(&mut h);
        h.get_by_label(l.text("desktop")).click();
        h.run();
        h.get_by_label(&format!("{} · {}", l.text("open"), l.text("terminal")))
            .click();
        settle(&mut h);
        terminal(&mut h, l, "ipconfig");
        let card = l
            .format(
                "term_ip_card",
                &[
                    ("id", "host-a-p0"),
                    ("address", "192.168.1.2"),
                    ("prefix", "24"),
                    ("mode", l.text("term_mode_static")),
                    ("mac", "02:00:00:00:00:01"),
                ],
            )
            .unwrap();
        let expected = [
            card.as_str(),
            l.text("term_gateway_none"),
            l.text("term_dns_none"),
        ]
        .join("\n");
        h.get_by_label(&expected);
        assert!(h.query_by_label_contains("\"interfaces\"").is_none());
        terminal(&mut h, l, "arp");
        h.get_by_label(l.text("term_arp_empty"));
        terminal(&mut h, l, "ping -c 1 192.168.1.3");
        let summary = l
            .format(
                "term_ping_summary_ok",
                &[("sent", "1"), ("received", "1"), ("lost", "0")],
            )
            .unwrap();
        // Stop as soon as ping reports: further steps would let the ARP cache expire.
        for _ in 0..60 {
            if h.query_by_label_contains(&summary).is_some() {
                break;
            }
            h.get_by_label(l.text("step")).click();
            settle(&mut h);
        }
        h.get_by_label_contains(&summary);
        let reply_prefix = l
            .format("term_ping_reply", &[("source", "192.168.1.3"), ("ms", "")])
            .unwrap();
        let reply_prefix = reply_prefix.split("  ").next().unwrap().trim_end();
        h.get_by_label_contains(reply_prefix);
        terminal(&mut h, l, "arp");
        let host_b_mac = h
            .state()
            .project()
            .devices
            .iter()
            .find(|d| d.id.as_str() == "host-b")
            .unwrap()
            .interfaces[0]
            .mac
            .to_string();
        h.get_by_label(
            &l.format(
                "term_arp_row",
                &[
                    ("address", "192.168.1.3"),
                    ("mac", &host_b_mac),
                    ("interface", "host-a-p0"),
                ],
            )
            .unwrap(),
        );
        #[cfg(feature = "ui-snapshots")]
        {
            h.snapshot(format!("j8-terminal-student-{}", l.code()));
            snapshots.extend_harness(&mut h);
        }
        h.state_mut().choose_level(Level::Expert);
        h.run();
        h.get_by_label_contains("\"interfaces\"");
        assert!(h.query_by_label(&expected).is_none());
        #[cfg(feature = "ui-snapshots")]
        {
            h.snapshot(format!("j8-terminal-expert-{}", l.code()));
            snapshots.extend_harness(&mut h);
        }
    }
}

#[test]
fn equipment_sheet_explains_fields_and_judges_the_gateway_while_typing() {
    #[cfg(feature = "ui-snapshots")]
    let mut snapshots = egui_kittest::SnapshotResults::new();
    for l in Language::ALL {
        let mut app = Workbench::from_project(routed(false)).unwrap();
        app.choose_language(l);
        let mut h = harness(app);
        h.run();
        h.get_by_label(&l.format("canvas_device", &[("label", "A")]).unwrap())
            .click();
        h.run();
        h.get_by_label(l.text("help_gateway"));
        h.get_by_label(l.text("help_ipv4"));
        h.get_by_label(l.text("advice_gateway_missing"));
        let gateway = |h: &mut Harness<'_, Workbench>, value: &str| {
            h.get_by_label(l.text("gateway"))
                .set_text_value_accesskit(value);
            h.run();
        };
        gateway(&mut h, "192.168.1.1");
        h.get_by_label(
            &l.format("advice_gateway_outside", &[("network", "192.168.10.0/24")])
                .unwrap(),
        );
        gateway(&mut h, "192.168.10");
        h.get_by_label(l.text("advice_gateway_invalid"));
        gateway(&mut h, "192.168.10.1");
        h.get_by_label(l.text("advice_gateway_ok"));
        #[cfg(feature = "ui-snapshots")]
        {
            h.snapshot(format!("j8-sheet-{}", l.code()));
            snapshots.extend_harness(&mut h);
        }
        // Advice reads the draft: nothing is applied until the existing Apply command.
        assert_eq!(h.state().project().devices[0].network.default_gateway, None);
        h.get_by_label(l.text("apply")).click();
        h.run();
        assert_eq!(
            h.state().project().devices[0]
                .network
                .default_gateway
                .as_deref(),
            Some("192.168.10.1")
        );
        h.state_mut().choose_level(Level::Expert);
        h.run();
        assert!(h.query_by_label(l.text("help_gateway")).is_none());
        h.get_by_label(l.text("advice_gateway_ok"));
    }
}

#[test]
fn a_mission_is_started_explained_repaired_scored_and_unlocks_the_next() {
    #[cfg(feature = "ui-snapshots")]
    let mut snapshots = egui_kittest::SnapshotResults::new();
    let catalog = sns_missions::builtin();
    let title = |i: usize, code: &str| sns_missions::text(&catalog[i].title, code).to_owned();
    for l in Language::ALL {
        let code = l.code();
        let mut app = Workbench::new().unwrap();
        app.choose_language(l);
        let mut h = harness(app);
        h.run();
        h.get_by_label(l.text("missions")).click();
        h.run();
        let start = |i: usize| format!("{} · {}", l.text("mission_start"), title(i, code));
        h.get_by_label(&start(0));
        assert!(h.get_by_label(&start(1)).accesskit_node().is_disabled());
        h.get_by_label(&start(0)).click();
        settle(&mut h);
        assert_eq!(h.state().project(), &catalog[0].project);
        let heading = l
            .format("mission_heading", &[("title", &title(0, code))])
            .unwrap();
        h.get_by_label(&heading);
        let objective = |i: usize, done: bool| {
            format!(
                "{} · {}",
                l.text(if done { "mission_done" } else { "mission_todo" }),
                sns_missions::text(&catalog[0].objectives[i].text, code)
            )
        };
        h.get_by_label(&objective(0, false));
        // "Check" explains the first unmet objective with the author's words.
        h.get_by_label(l.text("mission_check")).click();
        h.run();
        h.get_by_label(sns_missions::text(&catalog[0].objectives[0].explain, code));
        h.get_by_label(l.text("mission_hint_button")).click();
        h.run();
        h.get_by_label(&format!(
            "{} · {}",
            l.text("mission_hint"),
            sns_missions::text(&catalog[0].hints[0], code)
        ));
        // Repair with the guided cable, through the regular topology transaction.
        h.get_by_label(&format!("{} · C", l.text("cable_tool")))
            .click();
        h.run();
        for device in ["Poste C", "Commutateur"] {
            h.get_by_label(&l.format("canvas_device", &[("label", device)]).unwrap())
                .click();
            h.run();
        }
        h.get_by_label(&objective(0, true));
        // Once repaired, the explanation of that objective disappears.
        assert!(
            h.query_by_label(sns_missions::text(&catalog[0].objectives[0].explain, code))
                .is_none()
        );
        h.get_by_label(&format!("{} · C", l.text("cable_tool")))
            .click();
        h.run();
        h.get_by_label(&l.format("canvas_device", &[("label", "Poste A")]).unwrap())
            .click();
        h.run();
        h.get_by_label(&format!("{} · F3", l.text("mode_test")))
            .click();
        settle(&mut h);
        h.get_by_label(l.text("ping_target"))
            .set_text_value_accesskit("192.168.1.4");
        h.run();
        h.get_by_label(l.text("send_ping")).scroll_to_me();
        h.run();
        h.get_by_label(l.text("send_ping")).click();
        settle(&mut h);
        for _ in 0..40 {
            if h.query_by_label(l.text("mission_complete")).is_some() {
                break;
            }
            h.get_by_label(l.text("step")).click();
            settle(&mut h);
        }
        h.get_by_label(l.text("mission_complete"));
        h.get_by_label(&objective(1, true));
        h.get_by_label(&l.format("mission_stars", &[("count", "2")]).unwrap());
        assert_eq!(h.state().mission_stars("forgotten-cable"), 2);
        let report = h.state_mut().mission_report().unwrap();
        assert!(report.complete);
        assert_eq!(report.stars, Some(2));
        #[cfg(feature = "ui-snapshots")]
        {
            // Let transient cable highlights and packets finish before capturing.
            for _ in 0..4 {
                h.step();
            }
            h.run();
            h.snapshot(format!("j8-mission-complete-{code}"));
            snapshots.extend_harness(&mut h);
        }
        // Leaving Test resets the trace; the observed reply keeps the mission complete.
        h.get_by_label(&format!("{} · F2", l.text("mode_build")))
            .click();
        settle(&mut h);
        h.get_by_label(l.text("mission_complete"));
        h.get_by_label(l.text("mission_next")).click();
        h.run();
        // The repaired network is unsaved: the usual confirmation protects it.
        h.get_by_label(l.text("discard")).click();
        settle(&mut h);
        assert_eq!(h.state().project(), &catalog[1].project);
        h.get_by_label(
            &l.format("mission_heading", &[("title", &title(1, code))])
                .unwrap(),
        );
        h.get_by_label(l.text("missions")).click();
        h.run();
        assert!(!h.get_by_label(&start(1)).accesskit_node().is_disabled());
        #[cfg(feature = "ui-snapshots")]
        {
            h.snapshot(format!("j8-mission-map-{code}"));
            snapshots.extend_harness(&mut h);
        }
        // Close the map, which overlaps the card, then leave the mission.
        h.get_by_role_and_label(egui::Role::Button, l.text("missions"))
            .click();
        h.run();
        h.get_by_label(l.text("mission_quit")).click();
        h.run();
        assert!(h.state_mut().mission_report().is_none());
    }
}
