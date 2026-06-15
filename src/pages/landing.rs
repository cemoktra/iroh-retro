use leptos::{prelude::*, task::spawn_local};
use leptos_router::hooks::use_navigate;

#[component]
pub(crate) fn LandingPage() -> impl IntoView {
    let (session_id, set_session_id) = signal(String::new());
    let (is_loading, set_loading) = signal(false);

    let create_session = move |_| {
        let navigate = use_navigate();
        set_loading.set(true);

        let sk = iroh::SecretKey::generate();
        let pk = sk.public();
        let pkh = hex::encode(pk.to_vec());

        let global_node = use_context::<RwSignal<Option<crate::p2p::node::RetroNode>>>()
            .expect("Global node signal must be provided");

        spawn_local(async move {
            let server = match crate::p2p::node::RetroNode::host(sk).await {
                Ok(server) => server,
                Err(err) => {
                    tracing::error!("Failed to spawn node: {err}");
                    set_loading.set(false);
                    return;
                }
            };

            global_node.set(Some(server));

            navigate(&format!("/session/{}", pkh), Default::default());
        });
    };

    let join_session = move |_| {
        let navigate = use_navigate();
        let id = session_id.get();

        if !id.is_empty() {
            let clean_id = id.trim().to_string();
            navigate(&format!("/session/{}", clean_id), Default::default());
        }
    };

    view! {
        <div
            style:display="flex"
            style:justify-content="center"
            style:align-items="center"
            style="min-height: 100vh;"
        >
            <div class="retro-card" style:width="32rem">
                <div class="card-body">
                    <h2 class="card-title">P2P Retro</h2>
                    <p class="card-text">"Start a new retro session or join an existing session."</p>

                    <input
                        id="sessionId"
                        placeholder="Session ID"
                        on:input=move |ev| set_session_id.set(event_target_value(&ev))
                        prop:value=session_id
                    />

                    <div class="button-group">
                        <button
                            class="btn-custom"
                            on:click=create_session
                        >
                                {move || if is_loading.get() { "Creating..." } else { "Create Session" }}
                        </button>
                        <button
                            class="btn-custom"
                            disabled=move || session_id.get().is_empty()
                            on:click=join_session
                        >
                            "Join"
                        </button>
                    </div>
                </div>
            </div>
        </div>
    }
}
