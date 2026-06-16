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
                    <svg class="filled-icon" viewBox="0 0 98 96" xmlns="http://www.w3.org/2000/svg">
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
                    {move || if copy_success.get() {
                        view! {
                            <>
                                <svg class="nonfilled-icon" stroke-width="1.5" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">
                                    <path d="M10.5213 2.62368C11.3147 1.75255 12.6853 1.75255 13.4787 2.62368L14.4989 3.74391C14.8998 4.18418 15.4761 4.42288 16.071 4.39508L17.5845 4.32435C18.7614 4.26934 19.7307 5.23857 19.6757 6.41554L19.6049 7.92905C19.5771 8.52388 19.8158 9.10016 20.2561 9.50111L21.3763 10.5213C22.2475 11.3147 22.2475 12.6853 21.3763 13.4787L20.2561 14.4989C19.8158 14.8998 19.5771 15.4761 19.6049 16.071L19.6757 17.5845C19.7307 18.7614 18.7614 19.7307 17.5845 19.6757L16.071 19.6049C15.4761 19.5771 14.8998 19.8158 14.4989 20.2561L13.4787 21.3763C12.6853 22.2475 11.3147 22.2475 10.5213 21.3763L9.50111 20.2561C9.10016 19.8158 8.52388 19.5771 7.92905 19.6049L6.41553 19.6757C5.23857 19.7307 4.26934 18.7614 4.32435 17.5845L4.39508 16.071C4.42288 15.4761 4.18418 14.8998 3.74391 14.4989L2.62368 13.4787C1.75255 12.6853 1.75255 11.3147 2.62368 10.5213L3.74391 9.50111C4.18418 9.10016 4.42288 8.52388 4.39508 7.92905L4.32435 6.41553C4.26934 5.23857 5.23857 4.26934 6.41554 4.32435L7.92905 4.39508C8.52388 4.42288 9.10016 4.18418 9.50111 3.74391L10.5213 2.62368Z" stroke-width="1.5"></path>
                                    <path d="M9 12L11 14L15 10" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"></path>
                                </svg>
                                " Copied!"
                            </>
                        }.into_any()
                    } else {
                        view! {
                            <>
                                <svg class="nonfilled-icon" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">
                                    <path d="M18 22C19.6569 22 21 20.6569 21 19C21 17.3431 19.6569 16 18 16C16.3431 16 15 17.3431 15 19C15 20.6569 16 22 18 22Z" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"></path>
                                    <path d="M18 8C19.6569 8 21 6.65685 21 5C21 3.34315 19.6569 2 18 2C16.3431 2 15 3.34315 15 5C15 6.65685 16.3431 8 18 8Z" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"></path>
                                    <path d="M6 15C7.65685 15 9 13.6569 9 12C9 10.3431 7.65685 9 6 9C4.34315 9 3 10.3431 3 12C3 13.6569 4.34315 15 6 15Z" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"></path>
                                    <path d="M15.5 6.5L8.5 10.5" stroke-width="1.5"></path>
                                    <path d="M8.5 13.5L15.5 17.5" stroke-width="1.5"></path>
                                </svg>
                                " Share"
                            </>
                        }.into_any()
                    }}
                </button>

                {move || is_host.get().then(|| view! {
                    <button
                        class="retro-header-btn"
                        on:click=export_md
                        title="Export Markdown Summary"
                    >
                    <svg class="nonfilled-icon" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">
                        <path d="M6 20L18 20" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"></path>
                        <path d="M12 4V16M12 16L15.5 12.5M12 16L8.5 12.5" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"></path>
                    </svg>
                        " Summary"
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
                        <svg class="nonfilled-icon" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">
                            <path d="M5 20V19C5 15.134 8.13401 12 12 12V12C15.866 12 19 15.134 19 19V20" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"></path>
                            <path d="M12 12C14.2091 12 16 10.2091 16 8C16 5.79086 14.2091 4 12 4C9.79086 4 8 5.79086 8 8C8 10.2091 9.79086 12 12 12Z" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"></path>
                        </svg>
                            " " {move || my_name.get()}
                        </span>
                    }.into_any()
                }}

                <div style:position="relative">
                    <button
                        class="retro-header-btn"
                        on:click=move |_| set_show_participants.update(|v| *v = !*v)
                        title="Show participants"
                    >
                    <svg class="nonfilled-icon" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">
                        <path d="M1 20V19C1 15.134 4.13401 12 8 12V12C11.866 12 15 15.134 15 19V20" stroke-width="1.5" stroke-linecap="round"></path>
                        <path d="M13 14V14C13 11.2386 15.2386 9 18 9V9C20.7614 9 23 11.2386 23 14V14.5" stroke-width="1.5" stroke-linecap="round"></path>
                        <path d="M8 12C10.2091 12 12 10.2091 12 8C12 5.79086 10.2091 4 8 4C5.79086 4 4 5.79086 4 8C4 10.2091 5.79086 12 8 12Z" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"></path>
                        <path d="M18 9C19.6569 9 21 7.65685 21 6C21 4.34315 19.6569 3 18 3C16.3431 3 15 4.34315 15 6C15 7.65685 16.3431 9 18 9Z" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"></path>
                    </svg>
                        " " {move || participant_count()}
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
