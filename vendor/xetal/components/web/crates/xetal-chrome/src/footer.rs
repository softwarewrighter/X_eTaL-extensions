//! The footer, as the other live demos show it: copyright, license,
//! the repository, the literate documents, and where and when this
//! build was made (from build.rs).

use yew::prelude::*;

pub(crate) const REPOSITORY: &str = "https://github.com/softwarewrighter/X_eTaL";

fn sep() -> Html {
    html! { <span class="sep">{ "\u{00b7}" }</span> }
}

pub fn footer() -> Html {
    html! {
        <footer>
            <span>{ "Copyright (c) 2026 Michael A Wright" }</span>{ sep() }
            <span>{ "MIT License" }</span>{ sep() }
            <a href={REPOSITORY} target="_blank">{ "Repository" }</a>{ sep() }
            <a href="literate/index.html" target="_blank">{ "Literate docs" }</a>{ sep() }
            <a href="poster/index.html" target="_blank">{ "Syntax poster" }</a>{ sep() }
            <span title="Built on this host, from this commit, at this time">{ format!(
                "Build ({} {} {})",
                env!("BUILD_HOST"),
                env!("BUILD_SHA"),
                env!("BUILD_TIMESTAMP")
            ) }</span>
        </footer>
    }
}
