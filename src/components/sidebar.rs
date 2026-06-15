use std::collections::HashMap;

use iroh::EndpointId;
use leptos::prelude::*;

#[component]
pub fn Sidebar<C, M>(
    my_name: Signal<String>,
    peer_list: Signal<HashMap<EndpointId, String>>,
    copy_success: Signal<bool>,
    on_copy_link: C,
    on_open_modal: M,
) -> impl IntoView
where
    C: Fn(leptos::ev::MouseEvent) + Send + 'static,
    M: Fn() + Send + 'static,
{
    view! {
        <div class="retro-card" style:width="18rem" style:padding="1.5rem" style:background="#272e33" style:border-radius="6px" style:display="flex" style:flex-direction="column" style:gap="1.2rem">
            <div>
                <h4 style:margin="0 0 0.5rem 0" style:color="#859289" style:font-size="0.85rem" style:text-transform="uppercase">"Session"</h4>
                <button
                    on:click=on_copy_link
                    style:width="100%" style:background="#343f44" style:color="#dbbc7f" style:border="1px solid #4f5b58" style:padding="0.5rem" style:border-radius="4px" style:cursor="pointer" style:font-weight="bold"
                >
                    {move || if copy_success.get() { "✓ Copied link!" } else { "🔗 Copy link" }}
                </button>
            </div>

            <hr style:border="0" style:border-top="1px solid #343f44" style:margin="0" />

            <div>
                <h3 class="card-title" style:margin="0 0 0.5rem 0" style:font-size="1.1rem" style:color="#d3c6aa">"Peers"</h3>
                <ul style:list-style="none" style:padding="0" style:margin="0">
                    <li style:padding="0.6rem 0" style:border-bottom="1px solid #343f44" style:display="flex" style:justify-content="space-between" style:align-items="center" style:color="#d3c6aa">
                        <span>"👤 " {move || my_name.get()} <span style:font-size="0.8rem" style:color="#a7c080">" (You)"</span></span>
                        <button
                            on:click=move |_| on_open_modal()
                            style:background="none" style:border="none" style:cursor="pointer" style:font-size="1rem" title="Edit name"
                        >
                            "✏️"
                        </button>
                    </li>
                    {move || peer_list.get().into_iter().map(|(id, name)| {
                        view! {
                            <li style:padding="0.6rem 0" style:border-bottom="1px solid #343f44" title=id.to_string() style:color="#d3c6aa">
                                "👤 " {name}
                            </li>
                        }
                    }).collect::<Vec<_>>()}
                </ul>
            </div>
        </div>
    }
}
