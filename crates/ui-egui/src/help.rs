//! Where the User Guide is, and which page of it answers what is on screen.
//!
//! The guide is an mdBook under `docs/guide`, published to GitHub Pages by
//! `.github/workflows/guide.yml` and served at [`BASE`] by a rewrite in the website
//! (woodsketchweb's `vercel.json`). Help > User Guide opens the front of it; F1 opens the page
//! for the tool in hand; a `?` on a tray panel's header opens the page for that panel.
//!
//! Nothing here holds any help text. The pages are written -- and most of them generated -- in
//! the other repository's sense of "written": `crates/ui-egui/tests/guide.rs` builds the
//! reference pages from the app itself. What this module holds is only the addresses, and the
//! tests below keep them pointing at pages that exist.

use woodsketch_tools::ToolId;

/// The front page of the guide.
pub const BASE: &str = "https://woodsketch.net/help/";

/// The id the guide gives a tool's row, so F1 can land on the tool rather than the top of a
/// long table: `ToolId::PushPull` -> `tool-push-pull`.
///
/// `tests/guide.rs` writes the anchors into the page **from this same function**, so the id the
/// app jumps to and the id the page carries cannot drift apart.
pub fn tool_anchor(tool: ToolId) -> String {
    let mut anchor = String::from("tool");
    for letter in format!("{tool:?}").chars() {
        if letter.is_ascii_uppercase() {
            anchor.push('-');
            anchor.push(letter.to_ascii_lowercase());
        } else {
            anchor.push(letter);
        }
    }
    anchor
}

/// The page explaining the tool in hand.
pub fn for_tool(tool: ToolId) -> String {
    format!("{BASE}reference/tools.html#{}", tool_anchor(tool))
}

/// The id the guide gives a panel's section, so a `?` lands on that panel rather than the top
/// of the page: `Panel::EntityInfo` -> `panel-entity-info`. As with [`tool_anchor`], the
/// generator writes the page's ids from this same function.
pub fn panel_anchor(panel: crate::tray::Panel) -> String {
    let mut anchor = String::from("panel");
    for letter in format!("{panel:?}").chars() {
        if letter.is_ascii_uppercase() {
            anchor.push('-');
            anchor.push(letter.to_ascii_lowercase());
        } else {
            anchor.push(letter);
        }
    }
    anchor
}

/// The section of the guide's tray page explaining a panel. Every panel has one now, so this
/// is no longer an `Option`: the page is generated from `Panel::ALL`, and the `?` on a header
/// lands on that panel's own section rather than the top of the page.
pub(crate) fn for_panel(panel: crate::tray::Panel) -> String {
    format!("{BASE}reference/tray.html#{}", panel_anchor(panel))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The guide's source, so a page can be checked for without a built book.
    fn guide() -> std::path::PathBuf {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/guide/src")
    }

    #[test]
    fn a_tools_anchor_reads_as_its_name() {
        assert_eq!(tool_anchor(ToolId::Select), "tool-select");
        assert_eq!(tool_anchor(ToolId::PushPull), "tool-push-pull");
        assert_eq!(tool_anchor(ToolId::ThreePointArc), "tool-three-point-arc");
        assert!(for_tool(ToolId::TapeMeasure).ends_with("reference/tools.html#tool-tape-measure"));
    }

    /// Every address this module can hand out points at a page that is really in the guide.
    /// A `?` onto a 404 is worse than no `?`, and nothing else would notice a page being
    /// renamed -- the links inside the book are checked by `tests/guide.rs`, but these live out
    /// here in the app.
    #[test]
    fn every_page_the_app_links_to_is_in_the_guide() {
        let mut linked: Vec<String> = ToolId::ALL.into_iter().map(for_tool).collect();
        linked.extend(crate::tray::Panel::ALL.into_iter().map(for_panel));
        assert!(linked.len() > ToolId::ALL.len(), "no panel links to the guide any more");
        for url in linked {
            let page = url.strip_prefix(BASE).expect("a guide address");
            let page = page.split('#').next().expect("a page");
            let source = guide().join(page.replace(".html", ".md"));
            assert!(source.exists(), "the app links to {page}, which isn't in docs/guide/src");
        }
    }
}
