use super::StatementState;
use crate::app::{
    components::editor::{
        status::{StatusIndicator, StatusLevel},
        DiagnosticsSlot, InfoSlot, StatementEditor,
    },
    pages::editor::EditorContext,
    util::{hotkeys::use_hotkey, uid::Uid},
};
use indexmap::IndexMap;
use leptos::{prelude::*, reactive::graph::ReactiveNode};
use leptos_use::sync_signal;
use std::{cmp::max, fmt::Display, iter, ops::Deref, time::Duration};
use strum::IntoEnumIterator;
use tracing::{info, warn};
use yggdrasil_engine::{
    error::EngineError,
    rules::branch::BranchRule,
    util::{expr_list_starts_with, expr_maybe_list_starts_with},
};

/// An error a branch can have
#[derive(Clone, Debug, PartialEq)]
pub enum BranchError {
    /// The user has not selected a branch rule to use
    NoRuleSelected,
    /// Either the patent branch or a subbranch has no statements
    NoStatements,
    /// The root statement has some problem
    DependentStatementError,
    /// The root statement has something logically wrong with it, like an incorrect application of a rule
    Root(EngineError),
    /// This sub-branch was not decomposed correctly
    SubBranch(Uid),
    /// Both sub-branches were not decomposed correctly
    SubBranches,
}

impl Display for BranchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoRuleSelected => write!(f, "No branch rules selected"),
            Self::NoStatements => write!(f, "Root branch or at least one sub-branch is empty"),
            Self::DependentStatementError => write!(f, "Statement has parsing error"),
            Self::Root(engine_error) => write!(f, "Could not make branch: {}", engine_error),
            Self::SubBranch(_) => write!(f, "One sub-branch is not decomposed correctly"),
            Self::SubBranches => write!(f, "Neither sub-branch is decomposed correctly"),
        }
    }
}

/// State of a branch
#[derive(Clone, Copy, Debug)]
pub struct BranchState {
    uid: Signal<Uid>,
    statements: RwSignal<IndexMap<Uid, StatementState>>,
    parent: Option<Signal<Uid>>,
    is_active: Signal<bool>,
    branch_rule: RwSignal<Option<BranchRule>>,
    current_error: Signal<Option<(BranchError)>>,
    sub: RwSignal<Option<(RwSignal<BranchState>, RwSignal<BranchState>)>>,
}

impl BranchState {
    pub fn new(uid: Signal<Uid>, parent: Option<Signal<Uid>>) -> Self {
        let branch_rule: RwSignal<Option<BranchRule>> = Default::default();
        let statements: RwSignal<IndexMap<Uid, StatementState>> = Default::default();
        let sub: RwSignal<Option<(RwSignal<BranchState>, RwSignal<BranchState>)>> =
            Default::default();

        let decomposed_rule = Memo::new(move |_| {
            let rule = branch_rule.read();
            let root_statement = statements.read().last().map(|(_, v)| v.expr().read());
            Some(
                rule.as_ref()?
                    .decompose(root_statement?.as_ref()?.simplify()),
            )
        });

        let current_error = Memo::new(move |_| {
            let sub = sub.read().map(|(one, two)| (*one.read(), *two.read()));
            let root_statement = statements.read().last().map(|(_, v)| v.expr().read());

            match (*branch_rule.read(), sub, root_statement) {
                (_, _, Some(root_statement)) if root_statement.is_none() => {
                    Some(BranchError::DependentStatementError)
                }
                (None, Some(_), _) => Some(BranchError::NoRuleSelected),
                (Some(_), None, _) => Some(BranchError::NoStatements),
                (Some(_), Some((sub1, sub2)), Some(_))
                    if let Some(decomposed_rule) = decomposed_rule.read().deref() =>
                {
                    match decomposed_rule {
                        Ok((correct1, correct2)) => {
                            let sub1_statements = sub1.statements.read();
                            let sub1_iter = sub1_statements.iter().map(|(_, v)| v.expr().read());

                            let sub2_statements = sub2.statements.read();
                            let sub2_iter = sub2_statements.iter().map(|(_, v)| v.expr().read());

                            let one = expr_maybe_list_starts_with(sub1_iter.clone(), correct1);
                            let two = expr_maybe_list_starts_with(sub1_iter, correct2);

                            let three = expr_maybe_list_starts_with(sub2_iter.clone(), correct1);
                            let four = expr_maybe_list_starts_with(sub2_iter, correct2);

                            match (one, two, three, four) {
                                (true, _, _, true) | (_, true, true, _) => None,
                                (true, _, _, _) | (_, true, _, _) => {
                                    Some(BranchError::SubBranch(sub2.uid().get()))
                                }
                                (_, _, true, _) | (_, _, _, true) => {
                                    Some(BranchError::SubBranch(sub1.uid().get()))
                                }
                                _ => Some(BranchError::SubBranches),
                            }
                        }
                        Err(err) => Some(BranchError::Root(err.clone())),
                    }
                }
                _ => None,
            }
        });

        Self {
            uid,
            statements,
            parent,
            branch_rule,
            current_error: current_error.into(),
            sub,
            is_active: Signal::stored(parent.is_none()),
        }
    }

    fn track_active(mut self, ctx: EditorContext) -> Self {
        // controls whether is branch is marked as active or not
        // branches with no parents (should only be the root branch) are always active
        self.is_active = Memo::new(move |_| {
            self.statements.with(|statements| {
                self.sub.with(|sub| {
                    ctx.focused_statement.with(|focused_statement| {
                        if let Some(focused_statement) = focused_statement {
                            if statements.get(focused_statement).is_some() {
                                true
                            } else if let Some((one, two)) = sub {
                                one.with(|one| {
                                    two.with(|two| one.is_active.get() || two.is_active.get())
                                })
                            } else {
                                false
                            }
                        } else {
                            false
                        }
                    })
                })
            })
        })
        .into();

        self
    }

    pub fn add_statement_with_uuid(
        &self,
        ctx: &EditorContext,
        uid: Uid,
        before: bool,
    ) -> StatementState {
        let statement = StatementState::new(
            Memo::new({
                let focused_statement = ctx.focused_statement;
                let new_uuid = uid.clone();
                move |_| {
                    if let Some(focus) = focused_statement.get() {
                        focus == new_uuid
                    } else {
                        false
                    }
                }
            })
            .into(),
            Memo::new(|_| false).into(),
        );

        self.statements.update({
            let uid = uid.clone();
            |s| {
                ctx.focused_statement
                    .with_untracked(|focused_statement| match focused_statement {
                        Some(id) if s.get_index_of(id).is_some() => {
                            let idx = {
                                let mut idx = s.get_index_of(id).unwrap();
                                if !before && idx < s.len() {
                                    idx += 1;
                                }
                                idx
                            };

                            s.shift_insert(idx, uid, statement);
                        }
                        _ => {
                            s.insert(uid, statement);
                        }
                    });
            }
        });
        statement
    }

    pub fn add_statement(&self, ctx: &EditorContext, before: bool) -> (Uid, StatementState) {
        let uid = Uid::new();

        (uid.clone(), self.add_statement_with_uuid(ctx, uid, before))
    }

    pub fn del_statement(&self, uid: &Uid) -> Result<Option<Uid>, &str> {
        let mut x = Ok(None);
        self.statements.update(|s| {
            if s.len() > 1 {
                let i = s.get_index_of(uid);
                s.shift_remove(uid);
                if let Some(i) = i {
                    let i = max(0, i as isize - 1);
                    x = Ok(s.get_index(i as usize).map(|(k, _)| k.clone()))
                } else {
                    x = Ok(None)
                }
            } else {
                x = Err("Cannot delete last statement in branch");
            }
        });

        x
    }

    pub fn uid(&self) -> Signal<Uid> {
        self.uid
    }

    pub fn current_error(&self) -> Signal<Option<BranchError>> {
        self.current_error
    }
}

/// Component to render a branch, its statements, and its sub-branches
#[component]
pub fn Branch(
    branch: BranchState,
    #[prop(into)] on_new_branch: Callback<(Uid, BranchState)>,
    #[prop(into)] on_delete_branch: Callback<Uid>,
    #[prop(into)] on_new_statement: Callback<(Uid, StatementState)>,
    #[prop(into)] on_delete_statement: Callback<Uid>,
    #[prop(into)] on_focus_branch: Callback<Uid>,
    #[prop(into)] on_focus_statement: Callback<Uid>,
) -> impl IntoView {
    let ctx = use_context::<EditorContext>().unwrap();

    let showing_branch_rule_selector = RwSignal::new(false);

    use_hotkey("ctrl+b", move |ev| {
        if let (Some(focused_statement), Some((last_statement, _))) = (
            ctx.focused_statement.read().as_ref(),
            branch.statements.read().last(),
        ) {
            if focused_statement == last_statement {
                let mut showing_branch_rule_selector = showing_branch_rule_selector.write();

                *showing_branch_rule_selector = !(*showing_branch_rule_selector);
            }
        }
    })
    .unwrap();

    // // make new branches when showing branch rule selector
    Effect::watch(
        move || showing_branch_rule_selector.get(),
        move |showing_branch_rule_selector, _, _| {
            if *showing_branch_rule_selector {
                let make_thing = || {
                    RwSignal::new({
                        let s = BranchState::new(Signal::stored(Uid::new()), Some(branch.uid))
                            .track_active(ctx);
                        on_new_branch.run((branch.uid().get(), s));
                        let (n_u, n_s) = s.add_statement(&ctx, false);
                        on_new_statement.run((n_u, n_s));
                        s
                    })
                };
                branch.sub.set(Some((make_thing(), make_thing())));
            } else {
                branch.branch_rule.set(None);
                branch.sub.update(|s| {
                    if let Some((one, two)) = s.take() {
                        for v in [one, two] {
                            v.with_untracked(|v| {
                                for (statement_uid, _) in (v.statements.read_untracked()).iter() {
                                    on_delete_statement.run(statement_uid.clone());
                                }
                                on_delete_branch.run(v.uid().get())
                            });
                        }
                    }
                });
            }
        },
        false,
    );

    view! {
        <div class=move || {
            "flex flex-col gap-2 items-start pl-1 border-l-2 border-white".to_string()
                + if branch.is_active.get() { "" } else { " opacity-25" }
        }>
            <For
                each=move || branch.statements.get()
                key=|(id, _)| { id.clone() }
                children=move |(id, input)| {
                    let last_statement = Memo::new({
                        let id = id.clone();
                        move |_| { &id == branch.statements.read().last().unwrap().0 }
                    });
                    view! {
                        <StatementEditor
                            statement=input
                            on_focus=move || {
                                on_focus_branch.run(branch.uid.get());
                                on_focus_statement.run(id.clone());
                            }
                        >
                            <InfoSlot slot>
                                <Show when=move || {
                                    last_statement() && showing_branch_rule_selector()
                                }>
                                    // this forces the select to have the right value
                                    {move || queue_microtask(move || {
                                        branch.branch_rule.mark_dirty();
                                    })}
                                    <select
                                        on:change=move |ev| {
                                            let new_value = event_target_value(&ev);
                                            branch
                                                .branch_rule
                                                .set(
                                                    match BranchRule::try_from(new_value.as_str()) {
                                                        Ok(v) => Some(v),
                                                        Err(_) => None,
                                                    },
                                                );
                                        }

                                        prop:value=move || {
                                            branch
                                                .branch_rule
                                                .read()
                                                .map(|v| v.to_string())
                                                .unwrap_or("_".to_string())
                                        }
                                    >
                                        <option value="_" selected disabled>
                                            "Select branch rule"
                                        </option>
                                        {BranchRule::iter()
                                            .map(|rule| {
                                                view! {
                                                    <option value=rule.to_string()>{rule.to_string()}</option>
                                                }
                                            })
                                            .collect_view()}
                                    </select>
                                </Show>

                            </InfoSlot>

                            <DiagnosticsSlot slot>
                                <Show when=move || {
                                    last_statement() && showing_branch_rule_selector()
                                }>
                                    <Show when=move || {
                                        last_statement() && showing_branch_rule_selector()
                                    }>
                                        <StatusIndicator state=Signal::derive(move || match branch
                                            .current_error
                                            .get()
                                        {
                                            Some(_) => StatusLevel::Bad,
                                            None => StatusLevel::Good,
                                        }) />
                                    </Show>
                                </Show>
                            </DiagnosticsSlot>
                        </StatementEditor>
                    }
                }
            />
            {move || {
                branch
                    .sub
                    .with(|sub| {
                        if let Some((one, two)) = sub {
                            Some(
                                view! {
                                    <div class="flex flex-col gap-4 w-full">
                                        {[one, two]
                                            .into_iter()
                                            .map(|v| {
                                                let v = *v;
                                                let is_problematic = Memo::new(move |_| {
                                                    let this_uid = v.read().uid.read();
                                                    match branch.current_error.read().as_ref() {
                                                        Some(BranchError::SubBranches) => true,
                                                        Some(BranchError::SubBranch(uid)) => uid == this_uid.deref(),
                                                        _ => false,
                                                    }
                                                });
                                                view! {
                                                    <div class="flex flex-row">
                                                        <div class="-ml-1 w-10 border-t-2 border-white">
                                                            <Show when=is_problematic>
                                                                <div class="h-min">
                                                                    <StatusIndicator state=Signal::stored(StatusLevel::Bad) />
                                                                </div>
                                                            </Show>
                                                        </div>
                                                        <Branch
                                                            branch=v.get()
                                                            on_new_branch=on_new_branch
                                                            on_delete_branch=on_delete_branch
                                                            on_new_statement=on_new_statement
                                                            on_delete_statement=on_delete_statement
                                                            on_focus_branch=on_focus_branch
                                                            on_focus_statement=on_focus_statement
                                                        />
                                                    </div>
                                                }
                                            })
                                            .collect_view()}

                                    </div>
                                },
                            )
                        } else {
                            None
                        }
                    })
            }}
        </div>
    }
    .into_any()
}
