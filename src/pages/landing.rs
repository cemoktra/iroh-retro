use leptos::{prelude::*, task::spawn_local};
use leptos_router::hooks::use_navigate;

use crate::components::theme_toggle::ThemeToggle;

#[component]
pub(crate) fn LandingPage() -> impl IntoView {
    let (session_id, set_session_id) = signal(String::new());
    let (is_loading, set_loading) = signal(false);

    let create_session = move |_| {
        let navigate = use_navigate();
        set_loading.set(true);

        let sk = iroh::SecretKey::generate();
        let pk = sk.public();
        let pkh = hex::encode(pk.as_bytes());

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
            <div class="retro-header-actions">
                <ThemeToggle />
            </div>
        </header>
        <div
            style:display="flex"
            style:justify-content="center"
            style:align-items="center"
            style="min-height: calc(100vh - 3.5rem);"
        >
            <div class="retro-card" style:max-width="32rem">
                <div class="card-body">
                    <h2 class="card-title">Iroh Retro</h2>
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
