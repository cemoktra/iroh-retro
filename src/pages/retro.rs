use std::collections::{HashMap, HashSet};

use iroh::EndpointId;
use leptos::{prelude::*, task::spawn_local};
use leptos_router::hooks::{use_navigate, use_params};
use leptos_router::params::Params;
use serde::Deserialize;
use tokio_stream::StreamExt;

use crate::components::action_panel::ActionPanel;
use crate::components::header::RetroHeader;
use crate::components::retro_column::RetroColumn;
use crate::models::{ActionItem, RetroNoteItem, SessionState, TimerState};
use crate::p2p::node::{
    ActionItemCommand, ActionItemMutation, ConnectionEvent, NodeMessage, NoteCommand, NoteMutation,
    PeerCommand, RetroNode, SessionCommand, TimerCommand,
};

#[derive(Clone, Debug, Params, PartialEq, Deserialize)]
pub struct SessionParams {
    id: String,
}

fn get_or_create_name() -> String {
    // try to load name from storage
    if let Some(window) = web_sys::window()
        && let Ok(Some(storage)) = window.local_storage()
        && let Ok(Some(saved_name)) = storage.get_item("retro_username")
        && !saved_name.trim().is_empty()
    {
        return saved_name;
    }

    // otherwise generate a beautiful random bame
    let colors = [
        "yellow", "red", "blue", "green", "purple", "orange", "pink", "cyan",
    ];
    let shapes = [
        "circle", "square", "triangle", "diamond", "star", "hexagon", "cube",
    ];
    let random_color_idx = (js_sys::Math::random() * colors.len() as f64).floor() as usize;
    let random_shape_idx = (js_sys::Math::random() * shapes.len() as f64).floor() as usize;
    let new_name = format!("{} {}", colors[random_color_idx], shapes[random_shape_idx]);

    new_name
}

fn current_timestamp_ms() -> f64 {
    js_sys::Date::now()
}

fn timer_remaining_seconds(timer: &TimerState, now_ms: f64) -> u64 {
    let Some(started_at) = timer.running_since_ms else {
        return timer.remaining_seconds;
    };

    let elapsed_seconds = ((now_ms - started_at).max(0.0) / 1000.0).floor() as u64;
    timer.remaining_seconds.saturating_sub(elapsed_seconds)
}

fn normalized_timer_state(timer: &TimerState, now_ms: f64) -> TimerState {
    let remaining_seconds = timer_remaining_seconds(timer, now_ms);

    TimerState {
        duration_seconds: timer.duration_seconds,
        remaining_seconds,
        running_since_ms: if remaining_seconds == 0 {
            None
        } else {
            timer.running_since_ms
        },
    }
}

fn format_timer_display(timer: &TimerState, now_ms: f64) -> String {
    let remaining_seconds = timer_remaining_seconds(timer, now_ms);
    let minutes = remaining_seconds / 60;
    let seconds = remaining_seconds % 60;
    format!("{minutes:02}:{seconds:02}")
}

fn schedule_alarm_partial(
    ctx: &web_sys::AudioContext,
    start_time: f64,
    duration: f64,
    frequency: f32,
    gain_value: f32,
) {
    let Ok(oscillator) = ctx.create_oscillator() else {
        return;
    };
    let Ok(gain) = ctx.create_gain() else {
        return;
    };

    oscillator.set_type(web_sys::OscillatorType::Sine);
    oscillator.frequency().set_value(frequency);
    gain.gain().set_value(0.0);

    let _ = oscillator.connect_with_audio_node(&gain);
    let _ = gain.connect_with_audio_node(&ctx.destination());

    let attack_time = start_time + 0.01;
    let release_time = start_time + duration;
    let fade_time = (release_time - 0.18).max(attack_time + 0.02);

    let _ = gain.gain().set_value_at_time(0.0, start_time);
    let _ = gain
        .gain()
        .linear_ramp_to_value_at_time(gain_value, attack_time);
    let _ = gain
        .gain()
        .linear_ramp_to_value_at_time(gain_value * 0.35, fade_time);
    let _ = gain.gain().linear_ramp_to_value_at_time(0.0, release_time);

    let _ = oscillator.start_with_when(start_time);
    let _ = oscillator.stop_with_when(release_time + 0.02);
}

fn schedule_alarm_chime(
    ctx: &web_sys::AudioContext,
    start_time: f64,
    base_frequency: f32,
    gain_value: f32,
) {
    let partials = [
        (base_frequency, gain_value, 0.46),
        (base_frequency * 2.0, gain_value * 0.78, 0.46),
        (base_frequency * 3.0, gain_value * 0.48, 0.44),
        (base_frequency * 4.0, gain_value * 0.28, 0.42),
        (base_frequency * 5.0, gain_value * 0.16, 0.40),
    ];

    for (frequency, partial_gain, duration) in partials {
        schedule_alarm_partial(ctx, start_time, duration, frequency, partial_gain);
    }
}

fn play_timer_alarm() {
    if let Ok(ctx) = web_sys::AudioContext::new() {
        let now = ctx.current_time();
        let pattern = [
            (0.00, 2057.14, 0.020),
            (1.00, 2057.14, 0.020),
            (2.00, 2057.14, 0.020),
            (3.00, 2057.14, 0.020),
        ];

        for (offset, frequency, gain_value) in pattern {
            schedule_alarm_chime(&ctx, now + offset, frequency, gain_value);
        }
    }
}

#[derive(Copy, Clone)]
struct EventListenerContext {
    node_signal: RwSignal<Option<RetroNode>>,
    is_host: bool,
    set_peer_list: WriteSignal<HashMap<EndpointId, String>>,
    my_name: ReadSignal<String>,
    good_notes: Signal<Vec<RetroNoteItem>>,
    set_good_notes: WriteSignal<Vec<RetroNoteItem>>,
    bad_notes: Signal<Vec<RetroNoteItem>>,
    set_bad_notes: WriteSignal<Vec<RetroNoteItem>>,
    action_items: Signal<Vec<ActionItem>>,
    set_action_items: WriteSignal<Vec<ActionItem>>,
    timer_state: Signal<TimerState>,
    set_timer_state: WriteSignal<TimerState>,
}

#[component]
pub(crate) fn RetroPage() -> impl IntoView {
    let params = use_params::<SessionParams>();

    let (is_connected, set_connected) = signal(false);
    let (error_msg, set_error_msg) = signal(Option::<String>::None);

    let (my_name, set_my_name) = signal(get_or_create_name());
    let (peer_list, set_peer_list) = signal(HashMap::<EndpointId, String>::new());

    let (good_notes, set_good_notes) = signal(Vec::<RetroNoteItem>::new());
    let (bad_notes, set_bad_notes) = signal(Vec::<RetroNoteItem>::new());
    let (action_items, set_action_items) = signal(Vec::<ActionItem>::new());
    let (timer_state, set_timer_state) = signal(TimerState::default());
    let (timer_now_ms, set_timer_now_ms) = signal(current_timestamp_ms());
    let (timer_alarm_played, set_timer_alarm_played) = signal(false);
    let (is_host, set_is_host) = signal(false);

    let (good_input, set_good_input) = signal(String::new());
    let (bad_input, set_bad_input) = signal(String::new());

    let (is_modal_open, set_modal_open) = signal(false);
    let (modal_name_input, set_modal_name_input) = signal(String::new());
    let (copy_success, set_copy_success) = signal(false);

    let name_input_ref = NodeRef::<leptos::html::Input>::new();
    Effect::new(move |_| {
        if is_modal_open.get()
            && let Some(el) = name_input_ref.get()
        {
            leptos::leptos_dom::helpers::request_animation_frame(move || {
                let _ = el.focus();
                el.select();
            });
        }
    });

    gloo_timers::callback::Interval::new(1000, move || {
        set_timer_now_ms.set(current_timestamp_ms());
    })
    .forget();

    let global_node =
        use_context::<RwSignal<Option<RetroNode>>>().expect("Global node signal must be provided");

    Effect::new(move |_| {
        let now_ms = timer_now_ms.get();
        let current_timer = timer_state.get();
        let normalized_timer = normalized_timer_state(&current_timer, now_ms);

        if normalized_timer.remaining_seconds > 0 {
            if timer_alarm_played.get() {
                set_timer_alarm_played.set(false);
            }
            return;
        }

        if current_timer.duration_seconds == 0 || timer_alarm_played.get() {
            return;
        }

        if current_timer.running_since_ms.is_some() || current_timer.remaining_seconds == 0 {
            play_timer_alarm();
            set_timer_alarm_played.set(true);
        }
    });

    Effect::new(move |_| {
        if !is_host.get() {
            return;
        }

        let now_ms = timer_now_ms.get();
        let current_timer = timer_state.get();
        if current_timer.running_since_ms.is_none() {
            return;
        }

        let normalized_timer = normalized_timer_state(&current_timer, now_ms);
        if normalized_timer.remaining_seconds > 0 {
            return;
        }

        set_timer_state.set(normalized_timer.clone());
        if let Some(node) = global_node.get_untracked() {
            spawn_local(async move {
                node.broadcast(&NodeMessage::Timer(TimerCommand::Update {
                    timer: normalized_timer,
                }))
                .await;
            });
        }
    });

    Effect::new(move |_| {
        let Ok(p) = params.get() else { return };
        let session_id = p.id.clone();
        let host = global_node.with_untracked(|node| node.is_some());

        if host {
            set_is_host.set(true);
            set_connected.set(true);
            start_event_listener(EventListenerContext {
                node_signal: global_node,
                is_host: true,
                set_peer_list,
                my_name,
                good_notes: good_notes.into(),
                set_good_notes,
                bad_notes: bad_notes.into(),
                set_bad_notes,
                action_items: action_items.into(),
                set_action_items,
                timer_state: timer_state.into(),
                set_timer_state,
            });
            return;
        }

        spawn_local(async move {
            let Ok(pk_bytes) = hex::decode(&session_id) else {
                return;
            };
            let Ok(pk_array): Result<[u8; 32], _> = pk_bytes.try_into() else {
                return;
            };
            let Ok(host_pk) = iroh::PublicKey::from_bytes(&pk_array) else {
                return;
            };

            match RetroNode::connect(host_pk).await {
                Ok(client_node) => {
                    let own_id = client_node.endpoint_id;
                    global_node.set(Some(client_node));
                    set_connected.set(true);

                    start_event_listener(EventListenerContext {
                        node_signal: global_node,
                        is_host: false,
                        set_peer_list,
                        my_name,
                        good_notes: good_notes.into(),
                        set_good_notes,
                        bad_notes: bad_notes.into(),
                        set_bad_notes,
                        action_items: action_items.into(),
                        set_action_items,
                        timer_state: timer_state.into(),
                        set_timer_state,
                    });

                    gloo_timers::callback::Timeout::new(200, move || {
                        if let Some(node) = global_node.get_untracked() {
                            let current_name = my_name.get_untracked();
                            let host_endpoint_id = host_pk;
                            spawn_local(async move {
                                node.broadcast(&NodeMessage::Peer(PeerCommand {
                                    endpoint_id: own_id,
                                    name: current_name,
                                }))
                                .await;
                                node.send_to(
                                    host_endpoint_id,
                                    &NodeMessage::Session(SessionCommand::RequestState {
                                        requester_id: own_id,
                                    }),
                                )
                                .await;
                            });
                        }
                    })
                    .forget();
                }
                Err(err) => {
                    set_error_msg.set(Some(format!("Failed to connect: {err}")));
                }
            }
        });
    });

    let add_note = move |content: String, category: String| {
        let clean_content = content.trim().to_string();
        if clean_content.is_empty() {
            return;
        }

        let note_id = format!("{}-{}", js_sys::Math::random(), js_sys::Date::now());
        let author = my_name.get_untracked();
        let Some(node) = global_node.get_untracked() else {
            return;
        };
        let author_id = node.endpoint_id;

        let new_note = RetroNoteItem {
            id: note_id.clone(),
            content: clean_content.clone(),
            author: author.clone(),
            author_id,
            votes: 0,
            voted_peers: HashSet::new(),
        };

        if category == "good" {
            set_good_notes.update(|list| list.push(new_note));
            set_good_input.set(String::new());
        } else {
            set_bad_notes.update(|list| list.push(new_note));
            set_bad_input.set(String::new());
        }

        spawn_local(async move {
            node.broadcast(&NodeMessage::Note(NoteCommand {
                note_id,
                command: NoteMutation::Create {
                    content: clean_content,
                    category,
                    author,
                    author_id,
                },
            }))
            .await;
        });
    };

    let toggle_vote_for_note = move |note_id: String| {
        let Some(node) = global_node.get_untracked() else {
            return;
        };
        let my_id = node.endpoint_id;

        let internal_toggle = |list: &mut Vec<RetroNoteItem>| {
            if let Some(note) = list.iter_mut().find(|n| n.id == note_id) {
                if note.voted_peers.contains(&my_id) {
                    note.voted_peers.remove(&my_id);
                    note.votes = note.votes.saturating_sub(1);
                } else {
                    note.voted_peers.insert(my_id);
                    note.votes += 1;
                }
            }
        };

        set_good_notes.update(|list| internal_toggle(list));
        set_bad_notes.update(|list| internal_toggle(list));

        let nid = note_id.clone();
        spawn_local(async move {
            node.broadcast(&NodeMessage::Note(NoteCommand {
                note_id: nid,
                command: NoteMutation::ToggleVote { peer_id: my_id },
            }))
            .await;
        });
    };

    let change_name = move |new_name: String| {
        if new_name.is_empty() || new_name == my_name.get_untracked() {
            return;
        }

        if let Some(window) = web_sys::window()
            && let Ok(Some(storage)) = window.local_storage()
        {
            let _ = storage.set_item("retro_username", &new_name);
        }

        set_my_name.set(new_name.clone());

        if let Some(node) = global_node.get_untracked() {
            let node_id = node.endpoint_id;
            let rename = |list: &mut Vec<RetroNoteItem>| {
                for note in list.iter_mut() {
                    if note.author_id == node_id {
                        note.author = new_name.clone();
                    }
                }
            };
            set_good_notes.update(rename);
            set_bad_notes.update(|list| rename(list));
            spawn_local(async move {
                node.broadcast(&NodeMessage::Peer(PeerCommand {
                    endpoint_id: node_id,
                    name: new_name,
                }))
                .await;
            });
        }
    };

    let apply_name_change = move || {
        let new_name = modal_name_input.get_untracked().trim().to_string();
        set_modal_open.set(false);
        change_name(new_name);
    };

    let has_i_voted = move |note: &RetroNoteItem| {
        if let Some(node) = global_node.get_untracked() {
            note.voted_peers.contains(&node.endpoint_id)
        } else {
            false
        }
    };

    let is_my_note = move |note: &RetroNoteItem| {
        if let Some(node) = global_node.get_untracked() {
            note.author_id == node.endpoint_id
        } else {
            false
        }
    };

    let edit_note = move |note_id: String, new_content: String| {
        let update = |list: &mut Vec<RetroNoteItem>| {
            if let Some(note) = list.iter_mut().find(|n| n.id == note_id) {
                note.content = new_content.clone();
            }
        };
        set_good_notes.update(update);
        set_bad_notes.update(|list| update(list));

        if let Some(node) = global_node.get_untracked() {
            spawn_local(async move {
                node.broadcast(&NodeMessage::Note(NoteCommand {
                    note_id,
                    command: NoteMutation::Edit { new_content },
                }))
                .await;
            });
        }
    };

    let delete_note = move |note_id: String| {
        set_good_notes.update(|list| list.retain(|n| n.id != note_id));
        set_bad_notes.update(|list| list.retain(|n| n.id != note_id));

        if let Some(node) = global_node.get_untracked() {
            spawn_local(async move {
                node.broadcast(&NodeMessage::Note(NoteCommand {
                    note_id,
                    command: NoteMutation::Delete,
                }))
                .await;
            });
        }
    };

    let add_action = move |content: String| {
        let id = format!("{}-{}", js_sys::Math::random(), js_sys::Date::now());
        set_action_items.update(|list| {
            list.push(ActionItem {
                id: id.clone(),
                content: content.clone(),
            })
        });
        if let Some(node) = global_node.get_untracked() {
            spawn_local(async move {
                node.broadcast(&NodeMessage::ActionItem(ActionItemCommand {
                    id,
                    command: ActionItemMutation::Create { content },
                }))
                .await;
            });
        }
    };

    let delete_action = move |id: String| {
        set_action_items.update(|list| list.retain(|a| a.id != id));
        if let Some(node) = global_node.get_untracked() {
            spawn_local(async move {
                node.broadcast(&NodeMessage::ActionItem(ActionItemCommand {
                    id,
                    command: ActionItemMutation::Delete,
                }))
                .await;
            });
        }
    };

    let start_timer = move |duration_seconds: u64| {
        if !is_host.get_untracked() {
            return;
        }

        let next_timer = TimerState {
            duration_seconds,
            remaining_seconds: duration_seconds,
            running_since_ms: Some(current_timestamp_ms()),
        };
        set_timer_state.set(next_timer.clone());

        if let Some(node) = global_node.get_untracked() {
            spawn_local(async move {
                node.broadcast(&NodeMessage::Timer(TimerCommand::Update {
                    timer: next_timer,
                }))
                .await;
            });
        }
    };

    let stop_timer = move || {
        if !is_host.get_untracked() {
            return;
        }

        let next_timer = TimerState {
            running_since_ms: None,
            ..normalized_timer_state(&timer_state.get_untracked(), current_timestamp_ms())
        };
        set_timer_state.set(next_timer.clone());

        if let Some(node) = global_node.get_untracked() {
            spawn_local(async move {
                node.broadcast(&NodeMessage::Timer(TimerCommand::Update {
                    timer: next_timer,
                }))
                .await;
            });
        }
    };

    let timer_display =
        Signal::derive(move || format_timer_display(&timer_state.get(), timer_now_ms.get()));

    let copy_session_link = move |_| {
        if let Some(window) = web_sys::window()
            && let Ok(href) = window.location().href()
        {
            let navigator = window.navigator();
            let clipboard = navigator.clipboard();

            let _ = clipboard.write_text(&href);
            set_copy_success.set(true);
            gloo_timers::callback::Timeout::new(2000, move || {
                set_copy_success.set(false);
            })
            .forget();
        }
    };

    on_cleanup(move || {
        let mut extracted_node = None;
        global_node.update(|node| {
            extracted_node = node.take();
        });
        if let Some(node) = extracted_node {
            spawn_local(async move {
                node.close().await;
            });
        }
    });

    let go_gome = move |_| {
        let navigate = use_navigate();
        navigate("/", Default::default());
    };

    view! {
        <div class:retro-page-container=true style:background="var(--bg1)" style:min-height="100vh" style:color="var(--fg)" style:font-family="sans-serif">
            {move || if let Some(err) = error_msg.get() {
                view! {
                    <div class="alert alert-danger" style:text-align="center" style:margin-top="5rem">
                        <h3>"Failed to join"</h3>
                        <p>{err}</p>
                        <button
                            class="btn-custom"
                            on:click=go_gome
                        >
                            "Back"
                        </button>
                    </div>
                }.into_any()
            } else if is_connected.get() {
                view! {
                    <RetroHeader
                        my_name=my_name.into()
                        peer_list=peer_list.into()
                        on_name_change=change_name
                        copy_success=copy_success.into()
                        on_copy_link=copy_session_link
                        is_host=is_host.into()
                        good_notes=good_notes.into()
                        bad_notes=bad_notes.into()
                        action_items=action_items.into()
                        timer_state=timer_state.into()
                        timer_display=timer_display
                        on_timer_start=start_timer
                        on_timer_stop=stop_timer
                    />
                    <div class="retro-layout">

                        <div class="retro-columns">
                            <div class="retro-columns-inner">
                                <RetroColumn
                                    title="What went well".to_string()
                                    color="var(--green)".to_string()
                                    category="good".to_string()
                                    notes=good_notes.into()
                                    input_signal=good_input.into()
                                    on_input=move |val| set_good_input.set(val)
                                    on_add_note=add_note
                                    on_toggle_vote=toggle_vote_for_note
                                    has_voted=has_i_voted
                                    is_own=is_my_note
                                    on_edit=edit_note
                                    on_delete=delete_note
                                />

                                <RetroColumn
                                    title="What could be improved".to_string()
                                    color="var(--red)".to_string()
                                    category="bad".to_string()
                                    notes=bad_notes.into()
                                    input_signal=bad_input.into()
                                    on_input=move |val| set_bad_input.set(val)
                                    on_add_note=add_note
                                    on_toggle_vote=toggle_vote_for_note
                                    has_voted=has_i_voted
                                    is_own=is_my_note
                                    on_edit=edit_note
                                    on_delete=delete_note
                                />
                            </div>
                        </div>

                        <ActionPanel
                            is_host=is_host.into()
                            action_items=action_items.into()
                            on_add=add_action
                            on_delete=delete_action
                        />

                    </div>

                    {move || if is_modal_open.get() {
                        Some(view! {
                            <div style:position="fixed" style:top="0" style:left="0" style:width="100vw" style:height="100vh" style:background="rgba(30, 35, 38, 0.8)" style:display="flex" style:justify-content="center" style:align-items="center" style:z-index="999">
                                <div class="retro-modal-box" style:background="var(--bg2)" style:padding="2rem" style:border-radius="8px" style:border="1px solid var(--bg5)">
                                    <h3 style:margin-top="0" style:color="var(--fg)">"Edit name"</h3>
                                    <input
                                        type="text"
                                        node_ref=name_input_ref
                                        prop:value=modal_name_input
                                        on:input=move |ev| set_modal_name_input.set(event_target_value(&ev))
                                        on:keydown=move |ev| {
                                            if ev.key() == "Enter" {
                                                apply_name_change();
                                            }
                                        }
                                        style:width="100%" style:padding="0.5rem" style:background="var(--bg2)" style:color="var(--fg)" style:border="1px solid var(--bg5)" style:border-radius="4px" style:box-sizing="border-box" style:margin-bottom="1.5rem" style:font-size="1rem"
                                    />
                                    <div style:display="flex" style:justify-content="flex-end" style:gap="0.5rem">
                                        <button on:click=move |_| set_modal_open.set(false) style:background="var(--bg3)" style:color="var(--grey)" style:border="none" style:padding="0.5rem 1rem" style:border-radius="4px" style:cursor="pointer">
                                            "Cancel"
                                        </button>
                                        <button on:click=move |_| apply_name_change() style:background="var(--accent)" style:color="var(--bg2)" style:border="none" style:padding="0.5rem 1rem" style:border-radius="4px" style:cursor="pointer" style:font-weight="bold">
                                            "Save"
                                        </button>
                                    </div>
                                </div>
                            </div>
                        })
                    } else {
                        None
                    }}

                }.into_any()
            } else {
                view! {
                    <div class:loading-state=true style:text-align="center" style:margin-top="5rem">
                        <p style:font-size="1.2rem" style:color="var(--fg)">"Joining session..."</p>
                    </div>
                }.into_any()
            }}
        </div>
    }
}

fn start_event_listener(ctx: EventListenerContext) {
    let Some(node) = ctx.node_signal.get_untracked() else {
        return;
    };
    let mut stream = node.accept_events();

    spawn_local(async move {
        while let Some(msg) = stream.next().await {
            match msg {
                NodeMessage::Connection(ConnectionEvent::Accepted { endpoint_id: _ }) => {
                    let current_name = ctx.my_name.get_untracked();
                    let node_id = node.endpoint_id;
                    node.broadcast(&NodeMessage::Peer(PeerCommand {
                        endpoint_id: node_id,
                        name: current_name,
                    }))
                    .await;
                }
                NodeMessage::Peer(PeerCommand { endpoint_id, name }) => {
                    ctx.set_peer_list.update(|map| {
                        map.insert(endpoint_id, name.clone());
                    });
                    let rename = |list: &mut Vec<RetroNoteItem>| {
                        for note in list.iter_mut() {
                            if note.author_id == endpoint_id {
                                note.author = name.clone();
                            }
                        }
                    };
                    ctx.set_good_notes.update(rename);
                    ctx.set_bad_notes.update(rename);
                }
                NodeMessage::Note(NoteCommand {
                    note_id: id,
                    command:
                        NoteMutation::Create {
                            content,
                            category,
                            author,
                            author_id,
                        },
                }) => {
                    let item = RetroNoteItem {
                        id,
                        content,
                        author,
                        author_id,
                        votes: 0,
                        voted_peers: HashSet::new(),
                    };
                    if category == "good" {
                        ctx.set_good_notes.update(|list| list.push(item));
                    } else {
                        ctx.set_bad_notes.update(|list| list.push(item));
                    }
                }
                NodeMessage::Note(NoteCommand {
                    note_id,
                    command: NoteMutation::ToggleVote { peer_id },
                }) => {
                    let network_toggle = |list: &mut Vec<RetroNoteItem>| {
                        if let Some(note) = list.iter_mut().find(|n| n.id == note_id) {
                            if note.voted_peers.contains(&peer_id) {
                                note.voted_peers.remove(&peer_id);
                                note.votes = note.votes.saturating_sub(1);
                            } else {
                                note.voted_peers.insert(peer_id);
                                note.votes += 1;
                            }
                        }
                    };
                    ctx.set_good_notes.update(network_toggle);
                    ctx.set_bad_notes.update(network_toggle);
                }
                NodeMessage::Connection(ConnectionEvent::Closed { endpoint_id, .. }) => {
                    ctx.set_peer_list.update(|map| {
                        map.remove(&endpoint_id);
                    });
                }
                NodeMessage::Note(NoteCommand {
                    note_id,
                    command: NoteMutation::Edit { new_content },
                }) => {
                    let update = |list: &mut Vec<RetroNoteItem>| {
                        if let Some(note) = list.iter_mut().find(|n| n.id == note_id) {
                            note.content = new_content.clone();
                        }
                    };
                    ctx.set_good_notes.update(update);
                    ctx.set_bad_notes.update(update);
                }
                NodeMessage::Note(NoteCommand {
                    note_id,
                    command: NoteMutation::Delete,
                }) => {
                    ctx.set_good_notes
                        .update(|list| list.retain(|n| n.id != note_id));
                    ctx.set_bad_notes
                        .update(|list| list.retain(|n| n.id != note_id));
                }
                NodeMessage::ActionItem(ActionItemCommand {
                    id,
                    command: ActionItemMutation::Create { content },
                }) => {
                    ctx.set_action_items
                        .update(|list| list.push(ActionItem { id, content }));
                }
                NodeMessage::ActionItem(ActionItemCommand {
                    id,
                    command: ActionItemMutation::Delete,
                }) => {
                    ctx.set_action_items
                        .update(|list| list.retain(|a| a.id != id));
                }
                NodeMessage::Session(SessionCommand::RequestState { requester_id }) => {
                    if !ctx.is_host {
                        continue;
                    }

                    let state = SessionState {
                        good_notes: ctx.good_notes.get_untracked(),
                        bad_notes: ctx.bad_notes.get_untracked(),
                        action_items: ctx.action_items.get_untracked(),
                        timer: normalized_timer_state(
                            &ctx.timer_state.get_untracked(),
                            current_timestamp_ms(),
                        ),
                    };

                    node.send_to(
                        requester_id,
                        &NodeMessage::Session(SessionCommand::StateSnapshot {
                            requester_id,
                            state,
                        }),
                    )
                    .await;
                }
                NodeMessage::Session(SessionCommand::StateSnapshot {
                    requester_id,
                    state,
                }) => {
                    if requester_id != node.endpoint_id {
                        continue;
                    }

                    ctx.set_good_notes.set(state.good_notes);
                    ctx.set_bad_notes.set(state.bad_notes);
                    ctx.set_action_items.set(state.action_items);
                    ctx.set_timer_state
                        .set(normalized_timer_state(&state.timer, current_timestamp_ms()));
                }
                NodeMessage::Timer(TimerCommand::Update { timer }) => {
                    ctx.set_timer_state
                        .set(normalized_timer_state(&timer, current_timestamp_ms()));
                }
            }
        }
    });
}
