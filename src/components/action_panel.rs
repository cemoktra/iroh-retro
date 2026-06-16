use leptos::prelude::*;

use crate::models::ActionItem;

#[component]
pub fn ActionPanel<AddFn, DeleteFn>(
    is_host: Signal<bool>,
    action_items: Signal<Vec<ActionItem>>,
    on_add: AddFn,
    on_delete: DeleteFn,
) -> impl IntoView
where
    AddFn: Fn(String) + Clone + Send + 'static,
    DeleteFn: Fn(String) + Clone + Send + 'static,
{
    let (input, set_input) = signal(String::new());

    view! {
        <div class="retro-card retro-sidebar" style:padding="1.5rem" style:background="var(--bg2)" style:border-radius="6px" style:display="flex" style:flex-direction="column" style:gap="1rem">
            <h4 style:margin="0" style:color="var(--grey)" style:font-size="0.85rem" style:text-transform="uppercase" style:letter-spacing="0.05em">
                "Action Items"
            </h4>

            <ul style:list-style="none" style:padding="0" style:margin="0" style:display="flex" style:flex-direction="column" style:gap="0.5rem">
                {move || action_items.get().into_iter().map(|item| {
                    let item_id = item.id.clone();
                    let on_del = on_delete.clone();
                    view! {
                        <li style:display="flex" style:justify-content="space-between" style:align-items="flex-start" style:gap="0.5rem" style:padding="0.5rem 0" style:border-bottom="1px solid var(--bg3)">
                            <span style:color="var(--fg)" style:font-size="0.9rem" style:flex="1">{item.content}</span>
                            {is_host.get().then(|| view! {
                                <button
                                    on:click=move |_| on_del(item_id.clone())
                                    title="Delete action item"
                                    style:background="none" style:border="none" style:cursor="pointer" style:font-size="0.9rem" style:color="var(--grey)" style:flex-shrink="0" style:padding="0"
                                >
                                    <svg class="nonfilled-icon" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg" stroke-width="1.5">
                                        <path d="M20 9L18.005 20.3463C17.8369 21.3026 17.0062 22 16.0353 22H7.96474C6.99379 22 6.1631 21.3026 5.99496 20.3463L4 9"></path>
                                        <path d="M20 9L18.005 20.3463C17.8369 21.3026 17.0062 22 16.0353 22H7.96474C6.99379 22 6.1631 21.3026 5.99496 20.3463L4 9H20Z" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"></path>
                                        <path d="M21 6H15.375M3 6H8.625M8.625 6V4C8.625 2.89543 9.52043 2 10.625 2H13.375C14.4796 2 15.375 2.89543 15.375 4V6M8.625 6H15.375" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"></path>
                                    </svg>
                                </button>
                            })}
                        </li>
                    }
                }).collect::<Vec<_>>()}

                {move || action_items.get().is_empty().then(|| view! {
                    <li style:color="var(--bg5)" style:font-size="0.85rem" style:font-style="italic">"No action items yet."</li>
                })}
            </ul>

            {move || {
                let on_add_enter = on_add.clone();
                let on_add_click = on_add.clone();
                is_host.get().then(move || view! {
                    <div style:display="flex" style:gap="0.5rem" style:align-items="center">
                        <input
                            type="text"
                            placeholder="Add action item..."
                            prop:value=input
                            on:input=move |ev| set_input.set(event_target_value(&ev))
                            on:keydown=move |ev| {
                                if ev.key() == "Enter" {
                                    let val = input.get_untracked().trim().to_string();
                                    if !val.is_empty() {
                                        on_add_enter(val);
                                        set_input.set(String::new());
                                    }
                                }
                            }
                            style:flex="1" style:padding="0.4rem 0.6rem" style:margin="0" style:background="var(--bg2)" style:color="var(--fg)" style:border="1px solid var(--bg5)" style:border-radius="4px" style:font-size="0.9rem" style:box-sizing="border-box"
                        />
                        <button
                            on:click=move |_| {
                                let val = input.get_untracked().trim().to_string();
                                if !val.is_empty() {
                                    on_add_click(val);
                                    set_input.set(String::new());
                                }
                            }
                            style:background="var(--accent)" style:color="var(--bg2)" style:border="none" style:padding="0.4rem 0.7rem" style:border-radius="4px" style:cursor="pointer" style:font-weight="bold" style:font-size="0.9rem"
                        >
                            "+"
                        </button>
                    </div>
                })
            }}
        </div>
    }
}
