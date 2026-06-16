use leptos::prelude::*;

#[component]
pub fn ThemeToggle() -> impl IntoView {
    let initial_dark = web_sys::window()
        .and_then(|w| w.local_storage().ok().flatten())
        .and_then(|s| s.get_item("theme").ok().flatten())
        .map(|t| t != "light")
        .unwrap_or(true);
    let (dark_mode, set_dark_mode) = signal(initial_dark);

    Effect::new(move |_| {
        let dark = dark_mode.get();
        if let Some(window) = web_sys::window() {
            if let Some(doc) = window.document()
                && let Some(root) = doc.document_element()
            {
                if dark {
                    let _ = root.remove_attribute("data-theme");
                } else {
                    let _ = root.set_attribute("data-theme", "light");
                }
            }
            if let Some(storage) = window.local_storage().ok().flatten() {
                let _ = storage.set_item("theme", if dark { "dark" } else { "light" });
            }
        }
    });

    view! {
        <button
            class="retro-header-btn"
            on:click=move |_| set_dark_mode.update(|v| *v = !*v)
            title="Toggle theme"
        >
            {move || if dark_mode.get() {
                view! {
                    <>
                        <svg class="nonfilled-icon" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">
                            <path d="M12 18C15.3137 18 18 15.3137 18 12C18 8.68629 15.3137 6 12 6C8.68629 6 6 8.68629 6 12C6 15.3137 8.68629 18 12 18Z"  stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"></path>
                            <path d="M22 12L23 12" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"></path>
                            <path d="M12 2V1" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"></path>
                            <path d="M12 23V22" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"></path>
                            <path d="M20 20L19 19" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"></path>
                            <path d="M20 4L19 5" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"></path>
                            <path d="M4 20L5 19" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"></path>
                            <path d="M4 4L5 5" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"></path>
                            <path d="M1 12L2 12" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"></path>
                        </svg>
                        " Light"
                    </>
                }.into_any()
            } else {
                view! {
                    <>
                        <svg class="nonfilled-icon" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">
                            <path d="M3 11.5066C3 16.7497 7.25034 21 12.4934 21C16.2209 21 19.4466 18.8518 21 15.7259C12.4934 15.7259 8.27411 11.5066 8.27411 3C5.14821 4.55344 3 7.77915 3 11.5066Z" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"></path>
                        </svg>
                        " Dark"
                    </>
                }.into_any()
            }}
        </button>
    }
}
