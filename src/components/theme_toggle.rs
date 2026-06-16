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
            {move || if dark_mode.get() { "☀ Light" } else { "🌙 Dark" }}
        </button>
    }
}
