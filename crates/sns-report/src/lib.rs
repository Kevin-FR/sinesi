//! Frozen, bounded educational reports. No GUI, host I/O or simulation progression.
mod content;
mod pdf;
mod scene;
pub use content::{Report, ReportOptions, Snapshot};
pub use pdf::{PdfReport, render_pdf};
use sns_model::Diagnostic;
fn limit(target: &str) -> Diagnostic {
    Diagnostic::new("resource.limit_exceeded", target)
}
