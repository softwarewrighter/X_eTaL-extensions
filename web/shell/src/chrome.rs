//! Around every page: the header (logo, title, lede), panels and the
//! footer. Adapted from X_eTaL-demos' shared/microscope.

use yew::prelude::*;

pub const REPO: &str = "https://github.com/softwarewrighter/X_eTaL-extensions";

/// The logo (linking back to the catalog), the title and a lede.
pub fn header(title: &str, lede: &str) -> Html {
    html! {
        <header>
            <div class="brand">
                <a href="../" title="All extensions"><img class="logo" src="xetal-logo-red.png" alt="X_eTaL" /></a>
                <h1>{title}</h1>
            </div>
            <p class="lede">{lede}</p>
        </header>
    }
}

/// A panel: a title, a note, a body.
pub fn panel(title: &str, note: &str, body: Html) -> Html {
    html! {
        <section class="panel">
            <h2>{title}</h2>
            if !note.is_empty() { <p class="note">{note}</p> }
            {body}
        </section>
    }
}

fn sep() -> Html {
    html! { <span class="sep">{ "\u{00b7}" }</span> }
}

/// The footer: copyright, license, repository, the catalog, the
/// vendored X_eTaL commit and the build.
pub fn footer() -> Html {
    html! {
        <footer>
            <span>{ "Copyright (c) 2026 Michael A Wright" }</span>{ sep() }
            <span>{ "MIT License" }</span>{ sep() }
            <a href={REPO} target="_blank">{ "Repository" }</a>{ sep() }
            <a href="../">{ "All extensions" }</a>{ sep() }
            <span>{ format!("X_eTaL {}", env!("XETAL_SHA")) }</span>{ sep() }
            <span>{ format!("build (host {}, sha {}, {})", env!("BUILD_HOST"), env!("BUILD_SHA"), env!("BUILD_TIMESTAMP")) }</span>
        </footer>
    }
}
