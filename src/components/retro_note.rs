use leptos::prelude::*;

use crate::models::RetroNoteItem;

#[component]
pub fn RetroNote<V, H>(
    note: RetroNoteItem,
    color: String,
    on_toggle_vote: V,
    has_voted: H,
) -> impl IntoView
where
    V: Fn(String) + Clone + Send + 'static,
    H: Fn(&RetroNoteItem) -> bool + Clone + Send + 'static,
{
    let nid = note.id.clone();
    let voted = has_voted(&note);

    let bg_color = color.clone();
    let fg_color = color.clone();

    view! {
        <div style:background="#2e383c" style:padding="0.8rem" style:border-radius="6px" style:border-left=format!("3px solid {color}") style:display="flex" style:justify-content="space-between" style:align-items="center">
            <div>
                <p style:margin="0 0 0.4rem 0" style:color="#d3c6aa" style:word-break="break-word">{note.content}</p>
                <span style:font-size="0.75rem" style:color="#859289">"by " {note.author}</span>
            </div>
            <button
                on:click=move |_| on_toggle_vote(nid.clone())
                style:background=move || if voted { bg_color.clone() } else { "#343f44".to_string() }
                style:color=move || if voted { "#232a2e".to_string() } else { fg_color.clone() }
                style:border="1px solid #4f5b58" style:padding="0.3rem 0.6rem" style:border-radius="4px" style:font-weight="bold" style:display="flex" style:gap="0.4rem" style:align-items="center" style:cursor="pointer"
            >
                "👍 " <span>{note.votes}</span>
            </button>
        </div>
    }
}
