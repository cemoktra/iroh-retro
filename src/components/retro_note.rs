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
                            <svg class="nonfilled-icon" viewBox="0 0 24 24" stroke-width="1.5" xmlns="http://www.w3.org/2000/svg">
                                <path d="M14.3632 5.65156L15.8431 4.17157C16.6242 3.39052 17.8905 3.39052 18.6716 4.17157L20.0858 5.58579C20.8668 6.36683 20.8668 7.63316 20.0858 8.41421L18.6058 9.8942M14.3632 5.65156L4.74749 15.2672C4.41542 15.5993 4.21079 16.0376 4.16947 16.5054L3.92738 19.2459C3.87261 19.8659 4.39148 20.3848 5.0115 20.33L7.75191 20.0879C8.21972 20.0466 8.65806 19.8419 8.99013 19.5099L18.6058 9.8942M14.3632 5.65156L18.6058 9.8942" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"></path>
                            </svg>
                        </button>
                        <button
                            on:click=move |_| on_delete(id_delete.clone())
                            title="Delete note"
                            style:background="none" style:border="none" style:cursor="pointer" style:font-size="1rem" style:padding="0.1rem 0.2rem" style:color="var(--grey)"
                        >
                            <svg class="nonfilled-icon" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg" stroke-width="1.5">
                                <path d="M20 9L18.005 20.3463C17.8369 21.3026 17.0062 22 16.0353 22H7.96474C6.99379 22 6.1631 21.3026 5.99496 20.3463L4 9"></path>
                                <path d="M20 9L18.005 20.3463C17.8369 21.3026 17.0062 22 16.0353 22H7.96474C6.99379 22 6.1631 21.3026 5.99496 20.3463L4 9H20Z" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"></path>
                                <path d="M21 6H15.375M3 6H8.625M8.625 6V4C8.625 2.89543 9.52043 2 10.625 2H13.375C14.4796 2 15.375 2.89543 15.375 4V6M8.625 6H15.375" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"></path>
                            </svg>
                        </button>
                    }
                })}
                <button
                    on:click=move |_| on_toggle_vote(note_id_vote.clone())
                    style:background=move || if voted { bg_color.clone() } else { "var(--bg3)".to_string() }
                    style:color=move || if voted { "var(--bg2)".to_string() } else { "var(--fg)".to_string() }
                    style:border="1px solid var(--bg5)" style:padding="0.3rem 0.6rem" style:border-radius="4px" style:font-weight="bold" style:display="flex" style:gap="0.4rem" style:align-items="center" style:cursor="pointer"
                >
                    <svg class="yellow-icon" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">
                        <path fill-rule="evenodd" clip-rule="evenodd" d="M16.4724 20H4.1C3.76863 20 3.5 19.7314 3.5 19.4V9.6C3.5 9.26863 3.76863 9 4.1 9H6.86762C7.57015 9 8.22116 8.6314 8.5826 8.02899L11.293 3.51161C11.8779 2.53688 13.2554 2.44422 13.9655 3.33186C14.3002 3.75025 14.4081 4.30635 14.2541 4.81956L13.2317 8.22759C13.1162 8.61256 13.4045 9 13.8064 9H18.3815C19.7002 9 20.658 10.254 20.311 11.5262L18.4019 18.5262C18.1646 19.3964 17.3743 20 16.4724 20Z"/>
                    </svg>
                    <span>{note_votes}</span>
                </button>
            </div>
        </div>
    }
}
