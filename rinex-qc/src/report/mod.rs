use qc_traits::html_prelude::html;
use qc_traits::html_prelude::HtmlReport;
use qc_traits::html_prelude::RenderBox;
use qc_traits::html_prelude::*;

use std::collections::HashMap;

#[cfg(feature = "plots")]
mod plot;

#[cfg(feature = "plots")]
use plot::PlotItem;

/// HtmlContent describes what's inside [HtmlReport]
pub struct HtmlContent {
    /// Body (paragraph)
    body: Box<dyn HtmlReport>,
    #[cfg(feature = "plot")]
    /// Possible plot (after body/introduction)
    plot: Option<PlotItem>,
    /// Possible footnote (after body/introduction and plot)
    footnote: Box<dyn HtmlReport>,
}

impl HtmlReport for HtmlContent {
    fn to_html(&self) -> String {}
}

/// HtmlReport is used to report text and/or graphical analysis
pub struct nHtmlReport {
    /// Title (H1)
    title: String,
    /// Items, per div
    body: HashMap<String, HtmlContent>,
}

impl HtmlReport for nHtmlReport {
    fn georust_logo_url() -> &'static str {
        "https://raw.githubusercontent.com/georust/meta/master/logo/logo.png"
    }
    fn wiki_url() -> &'static str {
        "https://github.com/georust/rinex/wiki"
    }
    fn github_repo_url() -> &'static str {
        "https://github.com/georust/rinex"
    }
    fn to_html(&self) -> String {
        format!(
            "{}",
            html! {
                : doctype::HTML;
                html {
                    head {
                        meta(charset="utf-8");
                        meta(name="viewport", content="width=device-width, initial-scale=1");
                        link(rel="stylesheet", href="https:////cdn.jsdelivr.net/npm/bulma@0.9.4/css/bulma.min.css");
                        script(defer="true", src="https://use.fontawesome.com/releases/v5.3.1/js/all.js");
                        link(rel="icon", src=Self::georust_logo_url(), style="width:35px;height:35px;");
                    }
                    body {
                        div(id="title") {
                            img(src=Self::georust_logo_url(), style="width:100px;height:100px;") {}
                            h2(class="title") {
                                : self.title.clone()
                            }
                        }
                        div(id="wiki") {
                            table {
                                tr {
                                    br {
                                        a(href=Self::wiki_url()) {
                                            : "Online Documentation"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        )
    }
    fn to_inline_html(&self) -> Box<dyn RenderBox + '_> {
        box_html! {
            h1 {
                : self.title.to_string()
            }
            body {
                @ for (div, item) in &self.body {
                    div(name=div) {
                        : item.to_inline_html()
                    }
                }
            }
        }
    }
}
