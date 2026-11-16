//! Bounded presentation preferences, independent of project and execution data.
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WindowGeometry {
    pub x: i32,
    pub y: i32,
    pub width: u16,
    pub height: u16,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiFont {
    #[default]
    Default,
    NotoSans,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PanelLayout {
    #[default]
    InspectorRight,
    InspectorLeft,
    Tabs,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Representation {
    #[default]
    Icons,
    Diagram,
}
/// Commands offered by the workbench; never changes project or execution data.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Level {
    #[default]
    Student,
    Expert,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DisplayPreferences {
    pub scale_percent: u16,
    pub font: UiFont,
    pub representation: Representation,
    pub layout: PanelLayout,
    pub palette_width: u16,
    pub inspector_width: u16,
    pub window_width: u16,
    pub window_height: u16,
    pub application_scale_percent: u16,
    pub center_application_windows: bool,
    pub windows: BTreeMap<String, WindowGeometry>,
    /// Optional in profile version 2: an absent level means `student`.
    #[serde(default)]
    pub level: Level,
}
impl Default for DisplayPreferences {
    fn default() -> Self {
        Self {
            scale_percent: 100,
            font: UiFont::Default,
            representation: Representation::Icons,
            layout: PanelLayout::InspectorRight,
            palette_width: 185,
            inspector_width: 280,
            window_width: 1320,
            window_height: 880,
            application_scale_percent: 100,
            center_application_windows: false,
            windows: BTreeMap::new(),
            level: Level::Student,
        }
    }
}
impl DisplayPreferences {
    pub fn validate(&self) -> Result<(), &'static str> {
        if !(80..=200).contains(&self.scale_percent)
            || !(160..=360).contains(&self.palette_width)
            || !(240..=480).contains(&self.inspector_width)
            || !(840..=3840).contains(&self.window_width)
            || !(600..=2160).contains(&self.window_height)
            || !(75..=150).contains(&self.application_scale_percent)
        {
            return Err("settings.invalid_display");
        }
        if self.windows.len() > 16
            || self.windows.iter().any(|(key, rect)| {
                (sns_model::Software::parse(key).is_none() && key != "help")
                    || !(-16384..=16384).contains(&rect.x)
                    || !(-16384..=16384).contains(&rect.y)
                    || !(100..=7680).contains(&rect.width)
                    || !(60..=4320).contains(&rect.height)
            })
        {
            return Err("settings.invalid_display");
        }
        Ok(())
    }
}
impl DisplayPreferences {
    /// Persist only edits made against the effective view; temporary overrides stay temporary.
    pub fn copy_user_edits(&mut self, before: &Self, after: &Self) {
        macro_rules! fields {($($field:ident),+)=>{$(if before.$field != after.$field {self.$field=after.$field;})+};}
        fields!(
            scale_percent,
            font,
            representation,
            layout,
            palette_width,
            inspector_width,
            window_width,
            window_height,
            application_scale_percent,
            center_application_windows,
            level
        );
        for key in before.windows.keys().chain(after.windows.keys()) {
            if before.windows.get(key) != after.windows.get(key) {
                match after.windows.get(key) {
                    Some(value) => {
                        self.windows.insert(key.clone(), value.clone());
                    }
                    None => {
                        self.windows.remove(key);
                    }
                }
            }
        }
    }
}
