use std::collections::{HashMap, HashSet};

use iroh::EndpointId;
use leptos::{prelude::*, task::spawn_local};
use leptos_router::hooks::use_params;
use leptos_router::params::Params;
use serde::Deserialize;
use tokio_stream::StreamExt;

use crate::components::retro_column::RetroColumn;
use crate::components::sidebar::Sidebar;
use crate::models::RetroNoteItem;
use crate::p2p::node::{NodeMessages, RetroNode};

#[derive(Clone, Debug, Params, PartialEq, Deserialize)]
pub struct SessionParams {
    id: String,
}

fn get_or_create_name() -> String {
    // try to load name from storage
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            if let Ok(Some(saved_name)) = storage.get_item("retro_username") {
                if !saved_name.trim().is_empty() {
                    return saved_name;
                }
            }
        }
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

#[component]
pub(crate) fn RetroPage() -> impl IntoView {
    let params = use_params::<SessionParams>();

    let (is_connected, set_connected) = signal(false);
    let (error_msg, set_error_msg) = signal(Option::<String>::None);

    let (my_name, set_my_name) = signal(get_or_create_name());
    let (peer_list, set_peer_list) = signal(HashMap::<EndpointId, String>::new());

    let (good_notes, set_good_notes) = signal(Vec::<RetroNoteItem>::new());
    let (bad_notes, set_bad_notes) = signal(Vec::<RetroNoteItem>::new());

    let (good_input, set_good_input) = signal(String::new());
    let (bad_input, set_bad_input) = signal(String::new());

    let (is_modal_open, set_modal_open) = signal(false);
    let (modal_name_input, set_modal_name_input) = signal(String::new());
    let (copy_success, set_copy_success) = signal(false);

    let global_node =
        use_context::<RwSignal<Option<RetroNode>>>().expect("Global node signal must be provided");

    Effect::new(move |_| {
        let Ok(p) = params.get() else { return };
        let session_id = p.id.clone();
        let is_host = global_node.with_untracked(|node| node.is_some());

        if is_host {
            set_connected.set(true);
            start_event_listener(
                global_node,
                set_peer_list,
                my_name,
                set_good_notes,
                set_bad_notes,
            );
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

                    start_event_listener(
                        global_node,
                        set_peer_list,
                        my_name,
                        set_good_notes,
                        set_bad_notes,
                    );

                    gloo_timers::callback::Timeout::new(200, move || {
                        if let Some(node) = global_node.get_untracked() {
                            let current_name = my_name.get_untracked();
                            spawn_local(async move {
                                node.broadcast(&NodeMessages::Hello {
                                    endpoint_id: own_id,
                                    name: current_name,
                                })
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

        let new_note = RetroNoteItem {
            id: note_id.clone(),
            content: clean_content.clone(),
            author: author.clone(),
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

        if let Some(node) = global_node.get_untracked() {
            spawn_local(async move {
                node.broadcast(&NodeMessages::CreateNote {
                    id: note_id,
                    content: clean_content,
                    category,
                    author,
                })
                .await;
            });
        }
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
            node.broadcast(&NodeMessages::VoteToggle {
                note_id: nid,
                peer_id: my_id,
            })
            .await;
        });
    };

    let apply_name_change = move || {
        let new_name = modal_name_input.get_untracked().trim().to_string();
        let old_name = my_name.get_untracked();
        if new_name.is_empty() || new_name == old_name {
            set_modal_open.set(false);
            return;
        }

        // store new name
        if let Some(window) = web_sys::window() {
            if let Ok(Some(storage)) = window.local_storage() {
                let _ = storage.set_item("retro_username", &new_name);
            }
        }

        set_my_name.set(new_name.clone());
        set_modal_open.set(false);

        if let Some(node) = global_node.get_untracked() {
            let node_id = node.endpoint_id;
            spawn_local(async move {
                node.broadcast(&NodeMessages::NameUpdate {
                    endpoint_id: node_id,
                    old_name,
                    new_name,
                })
                .await;
            });
        }
    };

    let has_i_voted = move |note: &RetroNoteItem| {
        if let Some(node) = global_node.get_untracked() {
            note.voted_peers.contains(&node.endpoint_id)
        } else {
            false
        }
    };

    let copy_session_link = move |_| {
        if let Some(window) = web_sys::window() {
            let navigator = window.navigator();
            let clipboard = navigator.clipboard();

            if let Ok(href) = window.location().href() {
                let _ = clipboard.write_text(&href);
                set_copy_success.set(true);
                gloo_timers::callback::Timeout::new(2000, move || {
                    set_copy_success.set(false);
                })
                .forget();
            }
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

    view! {
        <div class:retro-page-container=true style:padding="2rem" style:background="#1e2326" style:min-height="100vh" style:color="#d3c6aa" style:font-family="sans-serif">
            {move || if let Some(err) = error_msg.get() {
                view! {
                    <div class="alert alert-danger">
                        <h3>"Failed to join"</h3>
                        <p>{err}</p>
                        <a href="/">"Back to landing page"</a>
                    </div>
                }.into_any()
            } else if is_connected.get() {
                view! {
                    <div style:display="flex" style:gap="2rem" style:align-items="flex-start">

                        <div style:flex="1" style:display="flex" style:flex-direction="column" style:gap="1.5rem">
                            <div style:display="flex" style:gap="1.5rem">
                                <RetroColumn
                                    title="What went well".to_string()
                                    color="#a7c080".to_string()
                                    category="good".to_string()
                                    notes=good_notes.into()
                                    input_signal=good_input.into()
                                    on_input=move |val| set_good_input.set(val)
                                    on_add_note=add_note.clone()
                                    on_toggle_vote=toggle_vote_for_note.clone()
                                    has_voted=has_i_voted
                                />

                                <RetroColumn
                                    title="What could be improved".to_string()
                                    color="#e67e80".to_string()
                                    category="bad".to_string()
                                    notes=bad_notes.into()
                                    input_signal=bad_input.into()
                                    on_input=move |val| set_bad_input.set(val)
                                    on_add_note=add_note.clone()
                                    on_toggle_vote=toggle_vote_for_note.clone()
                                    has_voted=has_i_voted
                                />
                            </div>
                        </div>

                        <Sidebar
                            my_name=my_name.into()
                            peer_list=peer_list.into()
                            copy_success=copy_success.into()
                            on_copy_link=copy_session_link
                            on_open_modal=move || {
                                set_modal_name_input.set(my_name.get_untracked());
                                set_modal_open.set(true);
                            }
                        />

                    </div>

                    {move || if is_modal_open.get() {
                        Some(view! {
                            <div style:position="fixed" style:top="0" style:left="0" style:width="100vw" style:height="100vh" style:background="rgba(26, 31, 34, 0.8)" style:display="flex" style:justify-content="center" style:align-items="center" style:z-index="999">
                                <div style:background="#272e33" style:padding="2rem" style:border-radius="8px" style:border="1px solid #4f5b58" style:width="22rem">
                                    <h3 style:margin-top="0" style:color="#d3c6aa">"Edit name"</h3>
                                    <input
                                        type="text"
                                        prop:value=modal_name_input
                                        on:input=move |ev| set_modal_name_input.set(event_target_value(&ev))
                                        on:keydown=move |ev| { if ev.key() == "Enter" { apply_name_change(); } }
                                        style:width="100%" style:padding="0.5rem" style:background="#2e383c" style:color="#d3c6aa" style:border="1px solid #4f5b58" style:border-radius="4px" style:box-sizing="border-box" style:margin-bottom="1.5rem" style:font-size="1rem"
                                    />
                                    <div style:display="flex" style:justify-content="flex-end" style:gap="0.5rem">
                                        <button on:click=move |_| set_modal_open.set(false) style:background="#343f44" style:color="#7a8478" style:border="none" style:padding="0.5rem 1rem" style:border-radius="4px" style:cursor="pointer">
                                            "Cancel"
                                        </button>
                                        <button on:click=move |_| apply_name_change() style:background="#a7c080" style:color="#232a2e" style:border="none" style:padding="0.5rem 1rem" style:border-radius="4px" style:cursor="pointer" style:font-weight="bold">
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
                        <p style:font-size="1.2rem" style:color="#d3c6aa">"Joining session..."</p>
                    </div>
                }.into_any()
            }}
        </div>
    }
}

fn start_event_listener(
    global_node: RwSignal<Option<RetroNode>>,
    set_peer_list: WriteSignal<HashMap<EndpointId, String>>,
    my_name: ReadSignal<String>,
    set_good_notes: WriteSignal<Vec<RetroNoteItem>>,
    set_bad_notes: WriteSignal<Vec<RetroNoteItem>>,
) {
    let Some(node) = global_node.get_untracked() else {
        return;
    };
    let mut stream = node.accept_events();

    spawn_local(async move {
        while let Some(msg) = stream.next().await {
            match msg {
                NodeMessages::Accepted { endpoint_id: _ } => {
                    let current_name = my_name.get_untracked();
                    let node_id = node.endpoint_id;
                    node.broadcast(&NodeMessages::Welcome {
                        endpoint_id: node_id,
                        name: current_name,
                    })
                    .await;
                }
                NodeMessages::Hello { endpoint_id, name } => {
                    set_peer_list.update(|map| {
                        map.insert(endpoint_id, name);
                    });
                    let current_name = my_name.get_untracked();
                    let node_id = node.endpoint_id;
                    node.broadcast(&NodeMessages::Welcome {
                        endpoint_id: node_id,
                        name: current_name,
                    })
                    .await;
                }
                NodeMessages::Welcome { endpoint_id, name } => {
                    set_peer_list.update(|map| {
                        map.insert(endpoint_id, name);
                    });
                }
                NodeMessages::NameUpdate {
                    endpoint_id,
                    new_name,
                    ..
                } => {
                    set_peer_list.update(|map| {
                        map.insert(endpoint_id, new_name);
                    });
                }
                NodeMessages::CreateNote {
                    id,
                    content,
                    category,
                    author,
                } => {
                    let item = RetroNoteItem {
                        id,
                        content,
                        author,
                        votes: 0,
                        voted_peers: HashSet::new(),
                    };
                    if category == "good" {
                        set_good_notes.update(|list| list.push(item));
                    } else {
                        set_bad_notes.update(|list| list.push(item));
                    }
                }
                NodeMessages::VoteToggle { note_id, peer_id } => {
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
                    set_good_notes.update(|list| network_toggle(list));
                    set_bad_notes.update(|list| network_toggle(list));
                }
                NodeMessages::Closed { endpoint_id, .. } => {
                    set_peer_list.update(|map| {
                        map.remove(&endpoint_id);
                    });
                }
            }
        }
    });
}
