//! The dividers between the panes: drag one to move its split (the
//! pointer is captured, so a fast drag is not lost), double-click it to
//! put the split back. The split is kept in this browser.

use web_sys::{Element, PointerEvent, Storage};
use yew::prelude::*;

use crate::{Axis, Split};

/// Where the split is kept, beside (not among) the saved files.
const KEY: &str = "xetal-layout";

fn storage() -> Option<Storage> {
    web_sys::window()?.local_storage().ok()?
}

fn save(split: Split) {
    if let Some(store) = storage() {
        let _ = store.set_item(KEY, &split.to_text());
    }
}

/// The split as this browser last left it.
#[hook]
pub fn use_split() -> UseStateHandle<Split> {
    use_state(|| {
        let saved = storage().and_then(|s| s.get_item(KEY).ok().flatten());
        saved.map_or_else(Split::default, |text| Split::from_text(&text))
    })
}

/// The divider that moves one split; it sits in the panes' grid, whose
/// box the pointer is measured against.
pub fn divider(axis: Axis, split: &UseStateHandle<Split>) -> Html {
    let grab = Callback::from(|e: PointerEvent| {
        let bar: Element = e.target_unchecked_into();
        let _ = bar.set_pointer_capture(e.pointer_id());
        e.prevent_default();
    });
    let s = split.clone();
    let drag = Callback::from(move |e: PointerEvent| {
        let bar: Element = e.target_unchecked_into();
        let Some(panes) = bar
            .parent_element()
            .filter(|_| bar.has_pointer_capture(e.pointer_id()))
        else {
            return;
        };
        let r = panes.get_bounding_client_rect();
        s.set(match axis {
            Axis::Columns => s.dragged(axis, e.client_x().into(), r.left(), r.width()),
            Axis::Rows => s.dragged(axis, e.client_y().into(), r.top(), r.height()),
        });
    });
    let kept = **split;
    let drop = Callback::from(move |_: PointerEvent| save(kept));
    let s = split.clone();
    let reset = Callback::from(move |_: MouseEvent| {
        s.set(Split::default());
        save(Split::default());
    });
    let (class, title) = match axis {
        Axis::Columns => (
            "divider columns",
            "Drag to widen a pane; double-click to even them",
        ),
        Axis::Rows => (
            "divider rows",
            "Drag to resize the output; double-click to reset",
        ),
    };
    html! {
        <div {class} {title} role="separator" onpointerdown={grab} onpointermove={drag}
            onpointerup={drop} ondblclick={reset}></div>
    }
}
