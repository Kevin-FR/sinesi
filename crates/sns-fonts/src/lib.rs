//! Offline document fonts shared by screen and export adapters (OFL-1.1 assets).
mod layout;
pub use layout::*;
pub const SANS: &[u8] = include_bytes!("../../../assets/fonts/NotoSans-Regular.ttf");
pub const SANS_BOLD: &[u8] = include_bytes!("../../../assets/fonts/NotoSans-Bold.ttf");
pub const MONO: &[u8] = include_bytes!("../../../assets/fonts/NotoSansMono-Regular.ttf");
pub const MONO_BOLD: &[u8] = include_bytes!("../../../assets/fonts/NotoSansMono-Bold.ttf");
pub const FONTS: [(&str, &[u8]); 4] = [
    ("sns-annotation-sans", SANS),
    ("sns-annotation-sans-bold", SANS_BOLD),
    ("sns-annotation-mono", MONO),
    ("sns-annotation-mono-bold", MONO_BOLD),
];
