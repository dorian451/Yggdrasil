mod components;
mod pages;
mod util;

use crate::error_template::{AppError, ErrorTemplate};
use components::header::Header;
use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::{components::*, path};
use pages::editor::Editor;
use util::hotkeys::provide_hotkeys_context;

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <AutoReload options=options.clone() />
                <HydrationScripts options />
                <MetaTags />
            </head>
            <body>
                <App />
            </body>
        </html>
    }
}

/// Main app entrypoint
#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    let main_ref = NodeRef::new();
    provide_hotkeys_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/yggdrasil.css" />

        <Html {..} class="w-full h-full" />
        <Body {..} class="flex flex-col w-full h-full" />

        <Title text="Yggdrasil" />

        <Router>
            <header>
                <Header />
            </header>
            <main class="flex flex-col flex-grow bg-zinc-700" node_ref=main_ref>
                <Routes fallback=|| {
                    let mut outside_errors = Errors::default();
                    outside_errors.insert_with_default_key(AppError::NotFound);
                    view! { <ErrorTemplate outside_errors /> }.into_view()
                }>
                    <Route path=path!("") view=Editor />
                </Routes>
            </main>
        </Router>
    }
}
