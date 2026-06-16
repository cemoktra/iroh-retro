use std::collections::HashMap;

use iroh::EndpointId;
use js_sys::Array;
use leptos::prelude::*;
use web_sys::wasm_bindgen::{JsCast, JsValue};

use crate::components::theme_toggle::ThemeToggle;
use crate::models::{ActionItem, RetroNoteItem, TimerState};

fn format_timer_input(duration_seconds: u64) -> String {
    let minutes = duration_seconds / 60;
    let seconds = duration_seconds % 60;

    if seconds == 0 {
        minutes.max(1).to_string()
    } else {
        format!("{minutes}:{seconds:02}")
    }
}

fn parse_timer_input(input: &str) -> Option<u64> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }

    if let Some((minutes, seconds)) = trimmed.split_once(':') {
        let minutes = minutes.trim().parse::<u64>().ok()?;
        let seconds = seconds.trim().parse::<u64>().ok()?;
        if seconds >= 60 {
            return None;
        }

        let total_seconds = minutes.saturating_mul(60).saturating_add(seconds);
        return (total_seconds > 0).then_some(total_seconds);
    }

    let minutes = trimmed.parse::<u64>().ok()?;
    let total_seconds = minutes.saturating_mul(60);
    (total_seconds > 0).then_some(total_seconds)
}

#[component]
pub fn RetroHeader<NameChangeFn, CopyLinkFn, StartTimerFn, StopTimerFn>(
    my_name: Signal<String>,
    peer_list: Signal<HashMap<EndpointId, String>>,
    on_name_change: NameChangeFn,
    copy_success: Signal<bool>,
    on_copy_link: CopyLinkFn,
    is_host: Signal<bool>,
    good_notes: Signal<Vec<RetroNoteItem>>,
    bad_notes: Signal<Vec<RetroNoteItem>>,
    action_items: Signal<Vec<ActionItem>>,
    timer_state: Signal<TimerState>,
    timer_display: Signal<String>,
    on_timer_start: StartTimerFn,
    on_timer_stop: StopTimerFn,
) -> impl IntoView
where
    NameChangeFn: Fn(String) + Clone + Send + 'static,
    CopyLinkFn: Fn(leptos::ev::MouseEvent) + Send + 'static,
    StartTimerFn: Fn(u64) + Clone + Send + 'static,
    StopTimerFn: Fn() + Clone + Send + 'static,
{
    let (editing, set_editing) = signal(false);
    let (edit_value, set_edit_value) = signal(String::new());
    let (show_participants, set_show_participants) = signal(false);
    let (timer_editing, set_timer_editing) = signal(false);
    let (timer_minutes_input, set_timer_minutes_input) = signal(String::new());

    let name_input_ref = NodeRef::<leptos::html::Input>::new();
    let timer_input_ref = NodeRef::<leptos::html::Input>::new();

    Effect::new(move |_| {
        if editing.get()
            && let Some(el) = name_input_ref.get()
        {
            leptos::leptos_dom::helpers::request_animation_frame(move || {
                let _ = el.focus();
                el.select();
            });
        }
    });

    Effect::new(move |_| {
        set_timer_minutes_input.set(format_timer_input(timer_state.get().duration_seconds));
    });

    Effect::new(move |_| {
        if timer_editing.get()
            && let Some(el) = timer_input_ref.get()
        {
            leptos::leptos_dom::helpers::request_animation_frame(move || {
                let _ = el.focus();
                el.select();
            });
        }
    });

    let participant_count = move || peer_list.get().len() + 1;

    let export_md = move |_| {
        let mut markdown = String::new();
        markdown.push_str("# Retro Export\n");

        markdown.push_str("## What went well\n");
        for note in good_notes.get_untracked() {
            markdown.push_str(&format!("- by {} with {} votes\n", note.author, note.votes));
            markdown.push_str("  ```\n");
            markdown.push_str(&format!("  {}\n", note.content));
            markdown.push_str("  ```\n");
        }

        markdown.push_str("## What could be improved\n");
        for note in bad_notes.get_untracked() {
            markdown.push_str(&format!("- by {} with {} votes\n", note.author, note.votes));
            markdown.push_str("  ```\n");
            markdown.push_str(&format!("  {}\n", note.content));
            markdown.push_str("  ```\n");
        }

        markdown.push_str("## Action Items\n");
        for item in action_items.get_untracked() {
            markdown.push_str(&format!("- {}\n", item.content));
        }

        if let Some(window) = web_sys::window()
            && let Some(document) = window.document()
        {
            let parts = Array::new();
            parts.push(&JsValue::from_str(&markdown));
            let opts = web_sys::BlobPropertyBag::new();
            opts.set_type("text/markdown");
            if let Ok(blob) = web_sys::Blob::new_with_str_sequence_and_options(&parts, &opts)
                && let Ok(url) = web_sys::Url::create_object_url_with_blob(&blob)
            {
                if let Ok(el) = document.create_element("a")
                    && let Some(anchor) = el.dyn_ref::<web_sys::HtmlAnchorElement>()
                {
                    anchor.set_href(&url);
                    anchor.set_download("retro-summary.md");
                    anchor.click();
                }
                let _ = web_sys::Url::revoke_object_url(&url);
            }
        }
    };

    view! {
        <header class="retro-header">
            <div class="retro-header-left">
                <a
                    href="https://github.com/cemoktra/iroh-retro"
                    target="_blank"
                    rel="noopener noreferrer"
                    class="retro-header-link"
                    title="GitHub"
                >
                    <svg class="retro-header-github-icon" viewBox="0 0 98 96" xmlns="http://www.w3.org/2000/svg">
                        <path fill-rule="evenodd" clip-rule="evenodd" d="M48.854 0C21.839 0 0 22 0 49.217c0 21.756 13.993 40.172 33.405 46.69 2.427.49 3.316-1.059 3.316-2.362 0-1.141-.08-5.052-.08-9.127-13.59 2.934-16.42-5.867-16.42-5.867-2.184-5.704-5.42-7.17-5.42-7.17-4.448-3.015.324-3.015.324-3.015 4.934.326 7.523 5.052 7.523 5.052 4.367 7.496 11.404 5.378 14.235 4.074.404-3.178 1.699-5.378 3.074-6.6-10.839-1.141-22.243-5.378-22.243-24.283 0-5.378 1.94-9.778 5.014-13.2-.485-1.222-2.184-6.275.486-13.038 0 0 4.125-1.304 13.426 5.052a46.97 46.97 0 0 1 12.214-1.63c4.125 0 8.33.571 12.213 1.63 9.302-6.356 13.427-5.052 13.427-5.052 2.67 6.763.97 11.816.485 13.038 3.155 3.422 5.015 7.822 5.015 13.2 0 18.905-11.404 23.06-22.324 24.283 1.78 1.548 3.316 4.481 3.316 9.126 0 6.6-.08 11.897-.08 13.526 0 1.304.89 2.853 3.316 2.364 19.412-6.52 33.405-24.935 33.405-46.691C97.707 22 75.788 0 48.854 0z"/>
                    </svg>
                </a>
                <span class="retro-header-title">"iroh-retro"</span>
            </div>

            <div class="retro-header-center">
                <div class="retro-timer">
                    {{
                        let on_timer_start_display = on_timer_start.clone();
                        move || {
                        if is_host.get() && timer_editing.get() {
                            let on_timer_start_enter = on_timer_start_display.clone();
                            view! {
                                <input
                                    type="text"
                                    node_ref=timer_input_ref
                                    class="retro-timer-input"
                                    prop:value=timer_minutes_input
                                    on:input=move |ev| set_timer_minutes_input.set(event_target_value(&ev))
                                    on:blur=move |_| set_timer_editing.set(false)
                                    on:keydown=move |ev| {
                                        match ev.key().as_str() {
                                            "Enter" => {
                                                if let Some(minutes) =
                                                    parse_timer_input(&timer_minutes_input.get_untracked())
                                                {
                                                    set_timer_editing.set(false);
                                                    on_timer_start_enter(minutes);
                                                }
                                            }
                                            "Escape" => set_timer_editing.set(false),
                                            _ => {}
                                        }
                                    }
                                    title="Timer duration, e.g. 5 or 2:30"
                                />
                            }
                            .into_any()
                        } else if is_host.get() {
                            view! {
                                <button
                                    class="retro-timer-display retro-timer-trigger"
                                    on:click=move |_| set_timer_editing.set(true)
                                    on:focus=move |_| set_timer_editing.set(true)
                                    title="Edit timer duration"
                                >
                                    {move || timer_display.get()}
                                </button>
                            }
                            .into_any()
                        } else {
                            view! {
                                <div class="retro-timer-display">{move || timer_display.get()}</div>
                            }
                            .into_any()
                        }
                    }
                    }}

                    {{
                        let on_timer_start_button = on_timer_start.clone();
                        let on_timer_stop_button = on_timer_stop.clone();
                        move || {
                        let on_timer_start_click = on_timer_start_button.clone();
                        let on_timer_stop_click = on_timer_stop_button.clone();

                        is_host.get().then(|| view! {
                            <>
                                <button
                                    class="retro-header-btn retro-timer-start"
                                    on:click=move |_| {
                                        if timer_state.get_untracked().running_since_ms.is_some() {
                                            on_timer_stop_click();
                                        } else if let Some(minutes) =
                                            parse_timer_input(&timer_minutes_input.get_untracked())
                                        {
                                            set_timer_editing.set(false);
                                            on_timer_start_click(minutes);
                                        }
                                    }
                                    title=move || {
                                        if timer_state.get().running_since_ms.is_some() {
                                            "Stop timer"
                                        } else {
                                            "Start timer"
                                        }
                                    }
                                >
                                    <span
                                        class=move || {
                                            if timer_state.get().running_since_ms.is_some() {
                                                "retro-timer-icon retro-timer-icon-stop"
                                            } else {
                                                "retro-timer-icon retro-timer-icon-play"
                                            }
                                        }
                                    />
                                </button>
                            </>
                        })
                    }
                    }}
                </div>
            </div>

            <div class="retro-header-actions">
                <button
                    class="retro-header-btn"
                    on:click=on_copy_link
                    title="Copy session link"
                >
                    {move || if copy_success.get() { "✓ Copied!" } else { "🔗 Copy link" }}
                </button>

                {move || is_host.get().then(|| view! {
                    <button
                        class="retro-header-btn"
                        on:click=export_md
                        title="Export Markdown Summary"
                    >
                        "⬇ Summary"
                    </button>
                })}

                {move || if editing.get() {
                    let on_nc_blur = on_name_change.clone();
                    let on_nc_enter = on_name_change.clone();
                    view! {
                        <input
                            type="text"
                            node_ref=name_input_ref
                            prop:value=edit_value
                            on:input=move |ev| set_edit_value.set(event_target_value(&ev))
                            on:blur=move |_| {
                                let val = edit_value.get_untracked().trim().to_string();
                                set_editing.set(false);
                                if !val.is_empty() { on_nc_blur(val); }
                            }
                            on:keydown=move |ev| {
                                match ev.key().as_str() {
                                    "Enter" => {
                                        let val = edit_value.get_untracked().trim().to_string();
                                        set_editing.set(false);
                                        if !val.is_empty() { on_nc_enter(val); }
                                    }
                                    "Escape" => set_editing.set(false),
                                    _ => {}
                                }
                            }
                            class="retro-header-name-input"
                        />
                    }.into_any()
                } else {
                    view! {
                        <span
                            class="retro-header-name"
                            title="Click to edit name"
                            on:click=move |_| {
                                set_edit_value.set(my_name.get_untracked());
                                set_editing.set(true);
                            }
                        >
                            "👤 " {move || my_name.get()}
                        </span>
                    }.into_any()
                }}

                <div style:position="relative">
                    <button
                        class="retro-header-btn"
                        on:click=move |_| set_show_participants.update(|v| *v = !*v)
                        title="Show participants"
                    >
                        "👥 " {move || participant_count()}
                    </button>

                    {move || show_participants.get().then(|| view! {
                        <div
                            style:position="fixed"
                            style:inset="0"
                            style:z-index="200"
                            on:click=move |_| set_show_participants.set(false)
                        />
                        <div class="retro-participants-dropdown">
                            <p class="retro-participants-title">"Participants"</p>
                            <ul class="retro-participants-list">
                                <li>
                                    {move || my_name.get()}
                                    <span class="retro-participants-you">" (You)"</span>
                                </li>
                                {move || peer_list.get().into_iter().map(|(id, name)| view! {
                                    <li title=id.to_string()>{name}</li>
                                }).collect::<Vec<_>>()}
                            </ul>
                        </div>
                    })}
                </div>

                <ThemeToggle />
            </div>
        </header>
    }
}
