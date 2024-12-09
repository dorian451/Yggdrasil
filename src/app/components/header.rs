use leptos::prelude::*;
use leptos_router::components::A;

/// The header that should be displayed at the top of every page
#[component]
pub fn Header() -> impl IntoView {
    view! {
        <div class="flex flex-grow p-4 bg-green-800">
            <A href="/" {..} class="flex gap-2 items-center">
                <img src="/logo.svg" class="h-16 aspect-square" />
                <h1 class="font-sans text-2xl font-bold text-white">"Yggdrasil"</h1>
            </A>
        </div>
    }
}
