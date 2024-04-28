//! Sets of Traits to describe analysis and processing of RINEX
//! and GNSS data more generaly.

// html report rendering
mod html;
pub use html::HtmlReport;

mod processing;
pub use processing::{Filter, MaskFilter, Masking, Preprocessing};
