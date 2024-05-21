//! Specific traits to generate RINEX and GNSS reports.
mod html;

pub enum Error {
    /// Html rendering error
    HtmlRendering,
}

pub mod html_prelude {
    pub use crate::html::HtmlReport;
    pub use horrorshow::{box_html, helper::doctype, html, RenderBox};
}
