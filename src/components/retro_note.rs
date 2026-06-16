use leptos::prelude::*;

use crate::models::RetroNoteItem;

#[component]
pub fn RetroNote<VoteFn, HasVotedFn, EditFn, DeleteFn>(
    note: RetroNoteItem,
    color: String,
    on_toggle_vote: VoteFn,
    has_voted: HasVotedFn,
    is_own: bool,
    on_edit: EditFn,
    on_delete: DeleteFn,
) -> impl IntoView
where
    VoteFn: Fn(String) + Clone + Send + 'static,
    HasVotedFn: Fn(&RetroNoteItem) -> bool + Clone + Send + 'static,
    EditFn: Fn(String, String) + Clone + Send + 'static,
    DeleteFn: Fn(String) + Clone + Send + 'static,
{
    let note_id = note.id.clone();
    let note_id_vote = note.id.clone();
    let note_id_delete = note.id.clone();
    let note_author = note.author.clone();
    let note_votes = note.votes;
    let note_content_initial = note.content.clone();
    let voted = has_voted(&note);

    let bg_color = color.clone();
    let fg_color = color.clone();

    let (editing, set_editing) = signal(false);
    let (edit_value, set_edit_value) = signal(note.content.clone());

    let input_ref = NodeRef::<leptos::html::Input>::new();

    Effect::new(move |_| {
        if editing.get()
            && let Some(el) = input_ref.get()
        {
            leptos::leptos_dom::helpers::request_animation_frame(move || {
                let _ = el.focus();
                el.select();
            });
        }
    });

    view! {
        <div style:background="var(--bg2)" style:padding="0.8rem" style:border-radius="6px" style:border-left=format!("3px solid {color}") style:display="flex" style:justify-content="space-between" style:align-items="center" style:gap="0.5rem">
            <div style:flex="1" style:min-width="0">
                {move || if editing.get() {
                    let on_edit_blur = on_edit.clone();
                    let on_edit_enter = on_edit.clone();
                    let id_blur = note_id.clone();
                    let id_enter = note_id.clone();
                    let content_escape = note_content_initial.clone();
                    view! {
                        <input
                            type="text"
                            node_ref=input_ref
                            prop:value=edit_value
                            on:input=move |ev| set_edit_value.set(event_target_value(&ev))
                            on:blur=move |_| {
                                let val = edit_value.get_untracked().trim().to_string();
                                set_editing.set(false);
                                if !val.is_empty() { on_edit_blur(id_blur.clone(), val); }
                            }
                            on:keydown=move |ev| {
                                match ev.key().as_str() {
                                    "Enter" => {
                                        let val = edit_value.get_untracked().trim().to_string();
                                        set_editing.set(false);
                                        if !val.is_empty() { on_edit_enter(id_enter.clone(), val); }
                                    }
                                    "Escape" => {
                                        set_edit_value.set(content_escape.clone());
                                        set_editing.set(false);
                                    }
                                    _ => {}
                                }
                            }
                            style:width="100%" style:padding="0.3rem 0.5rem" style:background="var(--bg1)" style:color="var(--fg)" style:border="1px solid var(--accent)" style:border-radius="4px" style:font-size="0.9rem" style:box-sizing="border-box"
                        />
                    }.into_any()
                } else {
                    let author = note_author.clone();
                    view! {
                        <p style:margin="0 0 0.4rem 0" style:color="var(--fg)" style:word-break="break-word">{move || edit_value.get()}</p>
                        <span style:font-size="0.75rem" style:color="var(--grey)">"by " {author}</span>
                    }.into_any()
                }}
            </div>
            <div style:display="flex" style:gap="0.4rem" style:align-items="center" style:flex-shrink="0">
                {is_own.then(|| {
                    let id_delete = note_id_delete;
                    view! {
                        <button
                            on:click=move |_| set_editing.update(|v| *v = !*v)
                            title="Edit note"
                            style:background="none" style:border="none" style:cursor="pointer" style:font-size="1rem" style:padding="0.1rem 0.2rem" style:color="var(--grey)"
                        >
                            "✏️"
                        </button>
                        <button
                            on:click=move |_| on_delete(id_delete.clone())
                            title="Delete note"
                            style:background="none" style:border="none" style:cursor="pointer" style:font-size="1rem" style:padding="0.1rem 0.2rem" style:color="var(--grey)"
                        >
                            "🗑️"
                        </button>
                    }
                })}
                <button
                    on:click=move |_| on_toggle_vote(note_id_vote.clone())
                    style:background=move || if voted { bg_color.clone() } else { "var(--bg3)".to_string() }
                    style:color=move || if voted { "var(--bg2)".to_string() } else { fg_color.clone() }
                    style:border="1px solid var(--bg5)" style:padding="0.3rem 0.6rem" style:border-radius="4px" style:font-weight="bold" style:display="flex" style:gap="0.4rem" style:align-items="center" style:cursor="pointer"
                >
                    "👍 " <span>{note_votes}</span>
                </button>
            </div>
        </div>
    }
}
