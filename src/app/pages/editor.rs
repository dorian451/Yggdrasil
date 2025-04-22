use std::{collections::HashMap, ops::Deref};

use crate::app::{
    components::editor::{
        branch::{Branch, BranchError, BranchState},
        StatementEditor, StatementError, StatementState, Toolbar, ToolbarEvent,
    },
    util::{hotkeys::use_hotkey, uid::Uid},
};
use indexmap::IndexMap;
use leptos::prelude::*;
use tracing::{info, warn};

/// Struct to contain the current editor state
#[derive(Clone, Copy, Debug)]
pub struct EditorContext {
    /// The currently focused branch in the editor
    pub focused_branch: RwSignal<Uid>,

    /// The currently focused statement in the editor
    pub focused_statement: RwSignal<Option<Uid>>,

    /// The root branch that all other branches are sub-branches of.
    pub root_branch: Signal<Uid>,

    /// Every branch currently in the editor
    pub branches: RwSignal<IndexMap<Uid, BranchState>>,

    /// Every statement currently in the editor
    pub statements: RwSignal<HashMap<Uid, StatementState>>,
}

fn handle_toolbar_event(ev: ToolbarEvent, ctx: &EditorContext) {
    match ev {
        ToolbarEvent::AddStatement { before } => {
            ctx.focused_branch.with_untracked(|focused_branch_uid| {
                ctx.branches.with_untracked(|branches| {
                    let branch = branches.get(focused_branch_uid).unwrap();

                    let (new_uid, new_statement) = branch.add_statement(ctx, before);
                    ctx.statements.update(|s| {
                        s.insert(new_uid.clone(), new_statement);
                    });
                    ctx.focused_statement.set(Some(new_uid));
                });
            });
        }
        ToolbarEvent::DeleteStatement => {
            ctx.focused_branch.with_untracked(|focused_branch_uid| {
                ctx.focused_statement.update(|focused_statement| {
                    ctx.branches.with_untracked(|branches| {
                        let branch = branches.get(focused_branch_uid).unwrap();

                        if let Some(id) = focused_statement {
                            let new_focus = branch.del_statement(id);
                            if let Ok(new_focus) = new_focus {
                                *focused_statement = new_focus;
                            }
                        }
                    });
                });
            });
        }
        _ => (),
    }
}

/// Component for the proof editor
#[component]
pub fn Editor() -> impl IntoView {
    // context
    provide_context({
        let root_branch_uid = Uid::new();
        let root_branch_state = BranchState::new(Signal::stored(root_branch_uid.clone()), None);

        let mut branches = IndexMap::new();
        branches.insert(root_branch_uid.clone(), root_branch_state);

        let mut ctx = EditorContext {
            focused_branch: RwSignal::new(root_branch_uid.clone()),
            focused_statement: RwSignal::default(),
            root_branch: RwSignal::new(root_branch_uid).read_only().into(),
            branches: RwSignal::new(branches),
            statements: Default::default(),
        };

        let mut statements = HashMap::new();
        let (one, two) = root_branch_state.add_statement(&ctx, false);
        statements.insert(one, two);
        ctx.statements = RwSignal::new(statements);

        ctx
    });

    let ctx = use_context::<EditorContext>().unwrap();

    // error message to display at the bottom of the page
    let current_message = Signal::derive(move || {
        let focused_branch = ctx.focused_branch.read();
        let focused_statement = ctx.focused_statement.read();

        let branch_problem = ctx
            .branches
            .read()
            .get(focused_branch.deref())
            .unwrap()
            .current_error()
            .read();

        let statement_problem = focused_statement
            .as_ref()
            .map(|v| ctx.statements.read().get(v).unwrap().current_error().read());

        match (branch_problem.as_ref(), statement_problem) {
            (Some(branch_problem), Some(ref s))
                if let Some(statement_problem) = s.as_ref()
                    && !matches!(branch_problem, BranchError::DependentStatementError) =>
            {
                Some(format!("{}\n{}", branch_problem, statement_problem))
            }
            (_, Some(ref s)) if let Some(statement_problem) = s.as_ref() => {
                Some(format!("{}", statement_problem))
            }
            (Some(branch_problem), _) => Some(format!("{}", branch_problem)),
            _ => None,
        }
    });

    // effects
    Effect::new(move |_| {
        ctx.focused_statement.with(|x| {
            info!("focused_statement: {:?}", x);
        })
    });

    // hotkeys
    use_hotkey("ctrl+a", move |_| {
        handle_toolbar_event(ToolbarEvent::AddStatement { before: false }, &ctx);
    })
    .unwrap();

    use_hotkey("ctrl+d", move |_| {
        handle_toolbar_event(ToolbarEvent::DeleteStatement, &ctx);
    })
    .unwrap();

    use_hotkey("ArrowUp", move |_| {
        // ctx.inputs.with(|inputs| {
        //     let i = inputs
        //         .get_index_of(&ctx.focused_statement.get_untracked().unwrap())
        //         .unwrap();
        //     if i > 0 {
        //         ctx.focused_statement
        //             .set(Some(inputs.get_index(i - 1).unwrap().0.clone()));
        //     }
        // })
    })
    .unwrap();

    use_hotkey("ArrowDown", move |_| {
        // ctx.inputs.with(|inputs| {
        //     let i = inputs
        //         .get_index_of(&ctx.focused_statement.get_untracked().unwrap())
        //         .unwrap();
        //     if i + 1 < inputs.len() {
        //         ctx.focused_statement
        //             .set(Some(inputs.get_index(i + 1).unwrap().0.clone()));
        //     }
        // })
    })
    .unwrap();

    view! {
        <Toolbar callback=move |ev| handle_toolbar_event(ev, &ctx) />
        <div class="flex flex-col gap-4 items-center pt-10 h-full">
            <h1 class="font-sans text-2xl font-bold text-white">"Untitled"</h1>
            <div class="flex flex-col gap-2 items-start w-4/5 h-full">
                <Branch
                    branch=ctx
                        .branches
                        .with_untracked(|b| {
                            ctx.root_branch.with_untracked(|root| { *(b.get(root).unwrap()) })
                        })
                    on_new_branch=Callback::new(move |(parent_uid, new_branch): (Uid, BranchState)| {
                        let idx = ctx.branches.read_untracked().get_index_of(&parent_uid).unwrap();
                        let mut branches = ctx.branches.write();
                        branches.shift_insert(idx, new_branch.uid().get_untracked(), new_branch);
                    })
                    on_delete_branch=Callback::new(move |uid| {
                        ctx.branches
                            .update(|b| {
                                b.shift_remove(&uid);
                            });
                    })
                    on_new_statement=Callback::new(move |(uid, statement)| {
                        ctx.statements
                            .update(move |s| {
                                info!("added {}", uid);
                                s.insert(uid, statement);
                            });
                    })
                    on_delete_statement=Callback::new(move |uid| {})
                    on_focus_branch=Callback::new(move |uid| { ctx.focused_branch.set(uid) })
                    on_focus_statement=Callback::new(move |uid| {
                        ctx.focused_statement.set(Some(uid));
                    })
                />
            </div>
            <div class="flex sticky bottom-0 flex-col justify-center items-center w-full text-white bg-black bg-opacity-40">
                {move || {
                    current_message
                        .get()
                        .map(|v| {
                            v.lines()
                                .map(|l| {
                                    view! { <p inner_html=l.to_owned() /> }
                                })
                                .collect_view()
                        })
                }}
            </div>

        </div>
    }
}
