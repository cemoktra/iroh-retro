use leptos::prelude::*;

use crate::components::retro_note::RetroNote;
use crate::models::RetroNoteItem;

#[component]
pub fn RetroColumn<AddNoteFn, VoteFn, HasVotedFn, InputFn, IsOwnFn, EditFn, DeleteFn>(
    title: String,
    color: String,
    category: String,
    notes: Signal<Vec<RetroNoteItem>>,
    input_signal: Signal<String>,
    on_input: InputFn,
    on_add_note: AddNoteFn,
    on_toggle_vote: VoteFn,
    has_voted: HasVotedFn,
    is_own: IsOwnFn,
    on_edit: EditFn,
    on_delete: DeleteFn,
) -> impl IntoView
where
    AddNoteFn: Fn(String, String) + Clone + Send + 'static,
    VoteFn: Fn(String) + Clone + Send + 'static,
    HasVotedFn: Fn(&RetroNoteItem) -> bool + Clone + Send + 'static,
    InputFn: Fn(String) + Clone + Send + 'static,
    IsOwnFn: Fn(&RetroNoteItem) -> bool + Clone + Send + 'static,
    EditFn: Fn(String, String) + Clone + Send + 'static,
    DeleteFn: Fn(String) + Clone + Send + 'static,
{
    let on_add_click = on_add_note.clone();
    let category_clone = category.clone();
    let col_color = color.clone();
    let on_input_clone = on_input.clone();
    let category_enter = category.clone();
    let on_add_enter = on_add_note.clone();

    view! {
        <div class="retro-card" style:flex="1" style:background="var(--bg2)" style:padding="1rem" style:border-top=format!("5px solid {color}") style:border-radius="6px">
            <h3 style:color=color.clone() style:margin-top="0">{title}</h3>

            <div style:display="flex" style:gap="0.5rem" style:margin-bottom="1rem" style:align-items="center">
                <input
                    type="text"
                    placeholder="Add card..."
                    prop:value=move || input_signal.get()
                    on:input=move |ev| on_input_clone(event_target_value(&ev))
                    on:keydown=move |ev| {
                        if ev.key() == "Enter" {
                            on_add_enter(input_signal.get_untracked(), category_enter.clone());
                        }
                    }
                    style:flex="1"
                    style:padding="0 0.6rem"
                    style:height="2.3rem"
                    style:background="var(--bg2)"
                    style:color="var(--fg)"
                    style:border="1px solid var(--bg5)"
                    style:border-radius="4px"
                    style:box-sizing="border-box"
                    style:font-size="0.95rem"
                />
                <button
                    on:click=move |_| on_add_click(input_signal.get_untracked(), category_clone.clone())
                    style:background=col_color.clone()
                    style:color="var(--bg2)"
                    style:border="none"
                    style:height="2.3rem"
                    style:aspect-ratio="1"
                    style:box-sizing="border-box"
                    style:padding="0"
                    style:border-radius="4px"
                    style:cursor="pointer"
                    style:font-weight="bold"
                    style:font-size="1.2rem"
                    style:display="flex"
                    style:align-items="center"
                    style:justify-content="center"
                >
                    "+"
                </button>
            </div>

            <div style:display="flex" style:flex-direction="column" style:gap="0.75rem">
                {move || notes.get().into_iter().map(|note| {
                    let own = is_own(&note);
                    view! {
                        <RetroNote
                            note=note
                            color=col_color.clone()
                            on_toggle_vote=on_toggle_vote.clone()
                            has_voted=has_voted.clone()
                            is_own=own
                            on_edit=on_edit.clone()
                            on_delete=on_delete.clone()
                        />
                    }
                }).collect::<Vec<_>>()}
            </div>
        </div>
    }
}
