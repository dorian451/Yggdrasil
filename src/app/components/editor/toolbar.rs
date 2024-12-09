use leptos::{either::Either, prelude::*};
use leptos_router::components::A;
use tracing::info;
use wasm_bindgen::JsCast;
use web_sys::HtmlElement;

/// Enum that lists what item the user clicked in the toolbar
#[derive(Clone, Copy, Debug)]
pub enum ToolbarEvent {
    NewFile,
    OpenFile,
    SaveFile,
    SaveFileAs,
    Quit,
    Undo,
    Redo,
    ToggleComment,
    TogglePremise,
    AddStatement {
        before: bool,
    },
    CreateBranch,
    DeleteStatement,
    DeleteBranch,
    Move {
        up: bool,
    },
    MoveBranch {
        up: bool,
    },
    MoveTree {
        up: bool,
    },
    BranchExpansion {
        /// 0 = toggle, 1 = expand, 2 = collapse, 3 = collapse terminated
        mode: u8,
    },
    EvaluationOptions,
    CheckStatement,
    CheckTree,
    ShortcutOptions,
    SubstitutionOptions,
    OpenUserGuide,
    OpenAbout,
    OpenBugReport,
    ToggleDevMode,
}

/// The toolbar to display at the top of the editor
#[component]
pub fn Toolbar<ToolbarF: Fn(ToolbarEvent) + Copy + 'static>(callback: ToolbarF) -> impl IntoView {
    // every item to display in the toolbar
    // use None for dividers
    let left_items: Vec<(&str, Vec<Option<(&str, ToolbarEvent)>>)> = vec![
        (
            "File",
            vec![
                Some(("New", ToolbarEvent::NewFile)),
                None,
                Some(("Open", ToolbarEvent::OpenFile)),
                None,
                Some(("Save", ToolbarEvent::SaveFile)),
                Some(("Save as...", ToolbarEvent::SaveFileAs)),
                None,
                Some(("Quit", ToolbarEvent::Quit)),
            ],
        ),
        (
            "Edit",
            vec![
                Some(("Undo", ToolbarEvent::Undo)),
                Some(("Redo", ToolbarEvent::Redo)),
                None,
                Some(("Toggle comment", ToolbarEvent::ToggleComment)),
                Some(("Toggle premise", ToolbarEvent::TogglePremise)),
                None,
                Some((
                    "Add statement before",
                    ToolbarEvent::AddStatement { before: true },
                )),
                Some((
                    "Add statement after",
                    ToolbarEvent::AddStatement { before: false },
                )),
                Some(("Create branch", ToolbarEvent::CreateBranch)),
                None,
                Some(("Delete statement", ToolbarEvent::DeleteStatement)),
                Some(("Delete branch", ToolbarEvent::DeleteBranch)),
            ],
        ),
        (
            "Navigate",
            vec![
                Some(("Move up", ToolbarEvent::Move { up: true })),
                Some(("Move down", ToolbarEvent::Move { up: false })),
                Some(("Move up branch", ToolbarEvent::MoveBranch { up: true })),
                Some(("Move down branch", ToolbarEvent::MoveBranch { up: false })),
                Some(("Move up tree", ToolbarEvent::MoveTree { up: true })),
                Some(("Move down tree", ToolbarEvent::MoveTree { up: false })),
                None,
                Some((
                    "Toggle branch expansion",
                    ToolbarEvent::BranchExpansion { mode: 0 },
                )),
                Some((
                    "Collapse all branches",
                    ToolbarEvent::BranchExpansion { mode: 2 },
                )),
                Some((
                    "Expand all branches",
                    ToolbarEvent::BranchExpansion { mode: 1 },
                )),
                Some((
                    "Collapse terminated branches",
                    ToolbarEvent::BranchExpansion { mode: 3 },
                )),
            ],
        ),
        (
            "Evaluate",
            vec![
                Some(("Options", ToolbarEvent::EvaluationOptions)),
                None,
                Some(("Check statement", ToolbarEvent::CheckStatement)),
                Some(("Check tree", ToolbarEvent::CheckTree)),
            ],
        ),
        (
            "Settings",
            vec![
                Some(("Shortcuts", ToolbarEvent::ShortcutOptions)),
                Some(("Substitutions", ToolbarEvent::SubstitutionOptions)),
            ],
        ),
        (
            "Help",
            vec![
                Some(("User guide", ToolbarEvent::OpenUserGuide)),
                None,
                Some(("About", ToolbarEvent::OpenAbout)),
                None,
                Some(("Report a bug", ToolbarEvent::OpenBugReport)),
                Some(("Toggle developer mode", ToolbarEvent::ToggleDevMode)),
            ],
        ),
    ];

    let links = vec![("Login", "auth/login"), ("Register", "auth/register")];

    let showing_menu = RwSignal::new(false);

    view! {
        <nav class="flex justify-items-start px-1 font-sans text-xs text-white bg-zinc-800">
            <label class="absolute top-0 left-0 z-40 w-full h-full has-[:checked]:hidden">
                <input
                    type="radio"
                    name="editor-toolbar"
                    class="hidden"
                    checked
                    id="editor-toolbar-clear"
                    on:change=move |_| { showing_menu.set(false) }
                />
            </label>
            //
            {left_items
                .into_iter()
                .map(|(name, items)| {
                    view! {
                        <div class="flex relative flex-col">
                            <label
                                class="z-50 py-1 px-2 select-none peer has-[:checked]:z-0 has-[:checked]:bg-zinc-500 hover:bg-zinc-500"

                                on:mouseover=move |e| {
                                    if showing_menu() {
                                        event_target::<HtmlElement>(&e).click();
                                    }
                                }
                            >
                                {name}
                                <input
                                    type="radio"
                                    name="editor-toolbar"
                                    class="hidden"
                                    on:change=move |_| { showing_menu.set(true) }
                                />
                            </label>
                            <div class="hidden absolute top-6 z-50 flex-col shadow-md peer-has-[:checked]:flex bg-zinc-600">
                                {items
                                    .into_iter()
                                    .map(|v| {
                                        if let Some((desc, action)) = v {
                                            Either::Left(
                                                view! {
                                                    <label
                                                        class="py-1 px-3 min-w-max select-none hover:bg-indigo-500"
                                                        for="editor-toolbar-clear"
                                                        on:click=move |_| { callback(action) }
                                                    >
                                                        {desc}
                                                    </label>
                                                },
                                            )
                                        } else {
                                            Either::Right(
                                                view! {
                                                    <div class="border-t border-white pointer-events-none" />
                                                },
                                            )
                                        }
                                    })
                                    .collect_view()}

                            </div>
                        </div>
                    }
                })
                .collect_view()}
            //
            //
            <div class="flex-grow" />
            {links
                .into_iter()
                .map(|(text, link)| {
                    view! {
                        <A href=|| { link.to_string() } {..} class="py-1 px-2 hover:bg-zinc-500">
                            {text}
                        </A>
                    }
                })
                .collect_view()}
        </nav>
    }
}
