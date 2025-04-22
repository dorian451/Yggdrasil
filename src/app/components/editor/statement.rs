use crate::app::{
    components::editor::status::{StatusIndicator, StatusLevel},
    util::hotkeys::use_hotkey,
};
use leptos::html::{HtmlElement, Input};
use leptos::prelude::*;
use leptos::{either::EitherOf3, ev::InputEvent};
use leptos_use::{core::IntoElementMaybeSignal, signal_debounced};
use std::{cell::LazyCell, collections::HashMap, fmt::Display, sync::LazyLock};
use strum::IntoEnumIterator;
use tracing::info;
use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;
use yggdrasil_engine::{error::EngineError, rules::branch::BranchRule};
use yggdrasil_grammar::{expr::Expr, Parser, PARSER};

/// An error a statement can have
#[derive(Clone, Debug, PartialEq)]
pub enum StatementError {
    /// The statement was not parsable
    Parsing(String),

    /// There is as logic problem with the statement
    Logic(EngineError),
}

impl Display for StatementError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Parsing(err) => write!(f, "Parsing error: {}", err),
            Self::Logic(engine_error) => {
                write!(f, "Evaluation error: {}", engine_error)
            }
        }
    }
}

/// The state of an individual statement
#[derive(Clone, Copy, Debug)]
pub struct StatementState {
    // the string input by the user
    raw: RwSignal<String>,
    // whatever error the statement has
    current_error: Signal<Option<StatementError>>,
    // the parsed expr tree, if the raw input was parsed without error
    expr: Signal<Option<Expr>>,
    // if the statement is currently focused in the editor
    focused: Signal<bool>,
    // if the statement is currently highlighted (being selected for the use of another statement's rule)
    highlighted: Signal<bool>,
}

impl StatementState {
    pub fn new(focused: Signal<bool>, highlighted: Signal<bool>) -> Self {
        let raw = RwSignal::new(String::new());

        let raw_debounced: Signal<String> = signal_debounced(raw, 100.0);
        let expr = Memo::new(move |_| {
            raw_debounced.with(|raw| {
                let raw = raw.clone();
                PARSER.with(move |parser| {
                    let parser = parser.get();
                    let res = parser.parse(&raw).into_result().map_err(|err| {
                        err.first()
                            .map(|v| {
                                format!(
                                    "chars: {}-{}: {:?}",
                                    v.span().start,
                                    v.span().end,
                                    v.reason()
                                )
                            })
                            .unwrap_or("Parse error".to_string())
                    });

                    res
                })
            })
        });

        let error = Memo::new(move |_| {
            let expr = expr.read();
            match expr.as_ref() {
                Ok(_) => None,
                Err(_) => Some(StatementError::Parsing("Invalid syntax".to_string())),
            }
        });

        Self {
            raw,
            current_error: error.into(),
            expr: Signal::derive(move || expr.get().ok()),
            focused,
            highlighted,
        }
    }

    pub fn expr(&self) -> &Signal<Option<Expr>> {
        &self.expr
    }
    pub fn current_error(&self) -> &Signal<Option<StatementError>> {
        &self.current_error
    }
}

/// Which characters to replace with symbols in the statement editor
static INPUT_MAPPINGS: LazyLock<HashMap<&str, &str>> = LazyLock::new(|| {
    HashMap::from_iter([
        ("^", "⊥"),
        ("~", "¬"),
        ("&", "∧"),
        ("|", "∨"),
        ("$", "→"),
        ("%", "↔"),
        ("@", "∀"),
        ("/", "∃"),
    ])
});

#[derive(Default)]
#[slot]
pub struct InfoSlot {
    #[prop(optional)]
    children: Option<Children>,
}

#[derive(Default)]
#[slot]
pub struct DiagnosticsSlot {
    #[prop(optional)]
    children: Option<Children>,
}

/// Component to show/edit an individual logic statement
#[component]
pub fn StatementEditor(
    statement: StatementState,
    #[prop(into)] on_focus: Callback<()>,
    #[prop(optional)] info_slot: InfoSlot,
    #[prop(optional)] diagnostics_slot: DiagnosticsSlot,
) -> impl IntoView {
    let input_box = NodeRef::<Input>::new();

    Effect::new(move |_| {
        if statement.focused.get() {
            if let Some(input_box) = input_box.get() {
                input_box.focus().unwrap();
            }
        }
    });

    view! {
        <div
            class="flex gap-2 w-full !bg-opacity-50 [&.focused]:bg-cyan-800"
            class:focused=statement.focused
            on:mousedown=move |_| on_focus.run(())
            on:focus=move |_| on_focus.run(())
        >
            <input
                node_ref=input_box
                type="text"
                on:keydown=move |ev| {
                    if INPUT_MAPPINGS.contains_key(ev.key().as_str()) {
                        ev.prevent_default();
                        let new_input = INPUT_MAPPINGS.get(ev.key().as_str()).unwrap();
                        let el = event_target::<HtmlInputElement>(&ev);
                        let selected_range = el
                            .selection_start()
                            .unwrap()
                            .map(|v| v as usize)
                            .unwrap_or(
                                statement.raw.get().len(),
                            )..el
                            .selection_end()
                            .unwrap()
                            .map(|v| v as usize)
                            .unwrap_or(statement.raw.get().len());
                        statement
                            .raw
                            .update(|v| {
                                let mut char_indices = v.char_indices();
                                let last_char_pos = match v.char_indices().last() {
                                    Some(last_char) => last_char.0 + last_char.1.len_utf8(),
                                    None => 0,
                                };
                                let char_start = char_indices
                                    .nth(selected_range.start)
                                    .map(|(x, _)| x)
                                    .unwrap_or(last_char_pos);
                                let char_end = if selected_range.end - selected_range.start == 0 {
                                    char_start
                                } else {
                                    char_indices
                                        .nth(selected_range.end - selected_range.start)
                                        .map(|(x, _)| x)
                                        .unwrap_or(last_char_pos)
                                };
                                v.replace_range(char_start..char_end, new_input);
                            });
                        let _ = el
                            .set_selection_range(
                                selected_range.start as u32 + 1,
                                selected_range.start as u32 + 1,
                            );
                    }
                }
                // on:input=move |ev| {
                // let ev = ev.dyn_ref::<InputEvent>().unwrap();
                // let new_txt = event_target_value(ev);
                // statement.raw.set(new_txt);
                // }
                on:focus=move |_| on_focus.run(())
                bind:value=statement.raw
            />

            {info_slot.children.map(|children| children())}

            <StatusIndicator state=Signal::derive(move || match statement.current_error.get() {
                Some(StatementError::Parsing(_)) => StatusLevel::Warn,
                Some(StatementError::Logic(_)) => StatusLevel::Bad,
                None => StatusLevel::Good,
            }) />

            {diagnostics_slot.children.map(|children| children())}
        </div>
    }
}
