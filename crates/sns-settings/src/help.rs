//! Offline help catalogue. It describes actions; it never executes them or opens files.
pub const PRODUCT_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const PROJECT_LICENSE: &str = include_str!("../../../LICENSE");
pub const FONT_LICENSE: &str = include_str!("../../../assets/fonts/OFL.txt");

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Topic {
    #[default]
    Design,
    Simulation,
    Documentation,
    Software,
    Examples,
    About,
}
impl Topic {
    pub const ALL: [Self; 6] = [
        Self::Design,
        Self::Simulation,
        Self::Documentation,
        Self::Software,
        Self::Examples,
        Self::About,
    ];
    pub fn label_key(self) -> &'static str {
        match self {
            Self::Design => "design",
            Self::Simulation => "simulation",
            Self::Documentation => "documentation",
            Self::Software => "software",
            Self::Examples => "help_examples",
            Self::About => "help_about",
        }
    }
    pub fn paragraph_keys(self) -> &'static [&'static str] {
        match self {
            Self::Design => &[
                "help_design_intro",
                "help_drag",
                "help_navigation",
                "help_keyboard",
                "help_shortcuts",
                "help_design_save",
            ],
            Self::Simulation => &[
                "help_simulation_intro",
                "help_simulation_steps",
                "help_simulation_tools",
                "help_simulation_observe",
                "help_simulation_limits",
            ],
            Self::Documentation => &[
                "help_documentation_intro",
                "help_documentation_edit",
                "help_documentation_export",
                "help_documentation_snapshot",
            ],
            Self::Software => &[
                "help_software_intro",
                "help_software_lifecycle",
                "help_software_files",
            ],
            Self::Examples => &["help_examples_intro"],
            Self::About => &["help_about_intro"],
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Example {
    Lan,
    Hosts,
    StaticRoutes,
    Rip,
    Classroom,
    Services,
    Private,
    Mail,
    P2p,
    Wireless,
    Direct,
    Arp,
    ModemListen,
    ModemConnect,
}
impl Example {
    pub const ALL: [Self; 14] = [
        Self::Lan,
        Self::Hosts,
        Self::Direct,
        Self::Arp,
        Self::Wireless,
        Self::Mail,
        Self::P2p,
        Self::Private,
        Self::Services,
        Self::Classroom,
        Self::StaticRoutes,
        Self::Rip,
        Self::ModemListen,
        Self::ModemConnect,
    ];

    pub fn project_bytes(self) -> &'static [u8] {
        match self {
            Self::Lan => include_bytes!("../../../fixtures/valid/lan.json"),
            Self::Hosts => include_bytes!("../../../fixtures/valid/hosts.json"),
            Self::Direct => include_bytes!("../../../fixtures/valid/direct.json"),
            Self::Arp => include_bytes!("../../../fixtures/valid/arp.json"),
            Self::Wireless => include_bytes!("../../../fixtures/valid/wireless.json"),
            Self::Mail => include_bytes!("../../../fixtures/valid/mail.json"),
            Self::P2p => include_bytes!("../../../fixtures/valid/p2p.json"),
            Self::Private => include_bytes!("../../../fixtures/valid/private.json"),
            Self::Services => include_bytes!("../../../fixtures/valid/services.json"),
            Self::Classroom => include_bytes!("../../../fixtures/valid/classroom.json"),
            Self::StaticRoutes => include_bytes!("../../../fixtures/valid/routed.json"),
            Self::Rip => include_bytes!("../../../fixtures/valid/rip.json"),
            Self::ModemListen => include_bytes!("../../../fixtures/valid/modem-listen.json"),
            Self::ModemConnect => include_bytes!("../../../fixtures/valid/modem-connect.json"),
        }
    }
    pub fn label_key(self) -> &'static str {
        match self {
            Self::Lan => "example",
            Self::Hosts => "software_example",
            Self::StaticRoutes => "static_example",
            Self::Rip => "rip_example",
            Self::Classroom => "classroom_example",
            Self::Services => "services_example",
            Self::Private => "private_example",
            Self::Mail => "mail_example",
            Self::P2p => "p2p_example",
            Self::Wireless => "wireless_example",
            Self::Direct => "direct_example",
            Self::Arp => "arp_example",
            Self::ModemListen => "modem_listen_example",
            Self::ModemConnect => "modem_connect_example",
        }
    }
    pub fn instructions_key(self) -> &'static str {
        match self {
            Self::Lan => "help_lesson_lan",
            Self::Hosts => "help_lesson_hosts",
            Self::StaticRoutes => "help_lesson_static",
            Self::Rip => "help_lesson_rip",
            Self::Classroom => "help_lesson_classroom",
            Self::Services => "help_lesson_services",
            Self::Private => "help_lesson_private",
            Self::Mail => "help_lesson_mail",
            Self::P2p => "help_lesson_p2p",
            Self::Wireless => "help_lesson_wireless",
            Self::Direct => "help_lesson_direct",
            Self::Arp => "help_lesson_arp",
            Self::ModemListen => "help_lesson_modem_listen",
            Self::ModemConnect => "help_lesson_modem_connect",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sns_i18n::Language;
    #[test]
    fn every_embedded_example_is_a_loadable_project() {
        for example in Example::ALL {
            assert!(
                sns_project::parse_project(example.project_bytes()).is_ok(),
                "{example:?}"
            );
        }
    }

    #[test]
    fn every_help_route_and_lesson_is_translated_in_all_six_languages() {
        for language in Language::ALL {
            for topic in Topic::ALL {
                assert!(language.try_text(topic.label_key()).is_some());
                for key in topic.paragraph_keys() {
                    assert!(
                        language.try_text(key).is_some(),
                        "{}:{key}",
                        language.code()
                    );
                }
            }
            for example in Example::ALL {
                assert!(language.try_text(example.label_key()).is_some());
                assert!(language.try_text(example.instructions_key()).is_some());
            }
        }
        assert!(!PRODUCT_VERSION.is_empty());
        assert!(PROJECT_LICENSE.contains("MIT License"));
        assert!(FONT_LICENSE.contains("SIL OPEN FONT LICENSE"));
    }
}
