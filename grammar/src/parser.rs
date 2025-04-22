use crate::expr::{constantexpr::{Constant, ConstantExpr}, Expr, literal::Literal, variable::Variable};
use chumsky::{
    extra::Full,
    prelude::{choice, just, map_ctx, recursive, regex, Rich},
    span::Span,
    Boxed, IterParser, Parser,
};
use std::{collections::HashMap, fmt::Debug};

pub(crate) type Input<'a> = &'a str;
pub(crate) type ContextType = HashMap<String, Variable>;
pub(crate) type Extras<'a> = Full<Rich<'a, char>, (), ContextType>;

fn grouping<'a, E, A: Parser<'a, Input<'a>, E, Extras<'a>> + Clone>(
    atom: A,
) -> impl Parser<'a, Input<'a>, E, Extras<'a>> + Clone {
    choice(
        [("(", ")"), ("[", "]"), ("{", "}")]
            .into_iter()
            .map(|(open, close)| {
                atom.clone()
                    .delimited_by(just(open).padded(), just(close).padded())
            })
            .collect::<Vec<_>>(),
    )
}

fn literal<'a>() -> impl Parser<'a, Input<'a>, Literal, Extras<'a>> + Clone {
    regex("[A-Z][a-zA-Z0-9]*").map(|v: &str| Literal(v.to_string()))
}

fn variable<'a>(new: bool) -> impl Parser<'a, Input<'a>, Variable, Extras<'a>> + Clone {
    regex("[t-z][a-zA-Z0-9]*").validate(move |v: &str, e, emitter| {
        let ctx: &ContextType = e.ctx();
        let found_var = ctx.get(v).cloned();
        if new {
            Variable {
                name: v.to_string(),
                id: nanoid::nanoid!(5),
            }
        } else {
            match found_var {
                Some(v) => v,
                None => {
                    emitter.emit(Rich::custom(e.span(), "This variable does not exist"));
                    Variable {
                        name: v.to_string(),
                        id: "**invalid**".to_string(),
                    }
                }
            }
        }
    })
}

fn tautology<'a>() -> impl Parser<'a, Input<'a>, Expr, Extras<'a>> + Clone {
    regex("⊤|1").to(Expr::Tautology)
}

fn contradiction<'a>() -> impl Parser<'a, Input<'a>, Expr, Extras<'a>> + Clone {
    regex("⊥|0").to(Expr::Contradiction)
}

fn predicate<'a>() -> impl Parser<'a, Input<'a>, Expr, Extras<'a>> + Clone {
    literal()
        .then(
            constant_expr()
                .separated_by(just(","))
                .at_least(1)
                .collect::<Vec<_>>()
                .delimited_by(just("("), just(")")),
        )
        .map(|(pred, args)| Expr::Predicate { pred, args })
}

fn constant<'a>() -> impl Parser<'a, Input<'a>, Constant, Extras<'a>> + Clone {
    regex("[a-s][a-zA-Z0-9]*").map(|v: &str| Constant(v.to_string()))
}

fn function<'a, T: Parser<'a, Input<'a>, ConstantExpr, Extras<'a>> + Clone>(
    atom: T,
) -> impl Parser<'a, Input<'a>, ConstantExpr, Extras<'a>> + Clone {
    constant()
        .then(
            atom.padded()
                .separated_by(just(","))
                .at_least(1)
                .collect::<Vec<_>>()
                .delimited_by(just("("), just(")")),
        )
        .map(|(func, args)| ConstantExpr::Function { func, args })
}

type InfixOpMap<T = Expr, R = T> = fn(Box<T>, Box<T>) -> R;

// used for infix operators like & and |, so that they all have the same precedence without associativity
fn infix_op_set<
    'a,
    E: Clone + Debug,
    R: Clone + 'a,
    T: Parser<'a, Input<'a>, E, Extras<'a>> + Clone,
    I: IntoIterator<
            Item = (
                Boxed<'a, 'a, Input<'a>, Input<'a>, Extras<'a>>,
                InfixOpMap<E, R>,
            ),
        > + Clone,
>(
    atom: T,
    things: I,
    fallback: R,
) -> impl Parser<'a, Input<'a>, R, Extras<'a>> + Clone {
    let any_op = choice(
        things
            .clone()
            .into_iter()
            .map(|(v, _)| v)
            .collect::<Vec<_>>(),
    );

    choice(
        things
            .into_iter()
            .map({
                |(op, into)| {
                    atom.clone()
                        .then((op.clone().to_span().rewind().then_ignore(op)).padded())
                        .then(atom.clone())
                        // attempt to capture extra usages of the operator and report them
                        // the operators this function was called with are not associative
                        .then(
                            (any_op.clone().to_span().then_ignore(atom.clone().or_not()))
                                .padded()
                                .repeated()
                                .collect::<Vec<_>>(),
                        )
                        .validate({
                            let fallback = fallback.clone();
                            move |(((a, op_span), b), invalid_input), _, emitter| {
                                if invalid_input.is_empty() {
                                    into(Box::new(a.clone()), Box::new(b))
                                } else {
                                    emitter.emit(
                                        Rich::custom(
                                            op_span.union(*invalid_input.last().unwrap()), 
                                            "The operators in this expression are not associative; use parentheses to indicate order of operation"
                                        )
                                    );

                                    fallback.clone()
                                }
                            }
                        })
                }
            })
            .collect::<Vec<_>>(),
    )
}

// used for universal and existential expressions
fn quantifier<
    'a,
    T: Parser<'a, Input<'a>, Expr, Extras<'a>> + Clone,
    M: Fn(Variable, Box<Expr>) -> Expr + Clone,
>(
    atom: T,
    sym: Input<'a>,
    map: M,
) -> impl Parser<'a, Input<'a>, Expr, Extras<'a>> + Clone {
    regex(sym)
        .ignore_then(variable(true).padded())
        .map_with(|v, e| {
            let mut new_ctx = e.ctx().clone();
            new_ctx.insert(v.name.clone(), v.clone());
            (new_ctx, v)
        })
        .then_with_ctx(map_ctx(|(ctx, _): &(ContextType, _)| ctx.clone(), atom))
        .map(move |((_, var), atom)| map(var, Box::new(atom)))
}

fn constant_expr_operator<'a>() -> impl Parser<'a, Input<'a>, &'a str, Extras<'a>> + Clone {
    regex(r"[^\w\s⊤⊥\(\)\[\]\{\}¬→↔∀@∃]{1,2}").padded()
}

fn constant_expr_atom<'a, T: Parser<'a, Input<'a>, ConstantExpr, Extras<'a>> + Clone>(
    const_expr: T,
) -> impl Parser<'a, Input<'a>, ConstantExpr, Extras<'a>> + Clone {
    choice((
        grouping(const_expr.clone()),
        function(const_expr.clone()),
        constant().map(ConstantExpr::Constant),
        variable(false).map(ConstantExpr::Variable),
        regex(r"[0-9]+").try_map(|digits: &str, span| {
            Ok(ConstantExpr::Number(digits.parse().map_err(|e| {
                Rich::custom(span, format!("Could not parse number: {}", e))
            })?))
        }),
    ))
}

fn constant_expr<'a>() -> impl Parser<'a, Input<'a>, ConstantExpr, Extras<'a>> + Clone {
    recursive(|const_expr| {
        let atom = constant_expr_atom(const_expr);

        choice((
            atom.clone()
                .then(constant_expr_operator())
                .then(atom.clone())
                .map(move |((a, op), b): ((_, &str), _)| {
                    ConstantExpr::Operator(op.to_string(), Box::new(a), Box::new(b))
                }),
            atom,
        ))
    })
    .padded()
}

pub fn parser<'a>() -> impl Parser<'a, Input<'a>, Expr, Extras<'a>> + Clone {
    recursive(|expr| {
        // expr parsers
        let atom = choice((
            grouping(expr),
            predicate(),
            literal().map(Expr::Literal),
            contradiction(),
            tautology(),
        ))
        .padded();

        // universal and existential
        // this is recursive since they can contain themselves (the other patterns cannot *directly* parse themselves)
        let atom = recursive(|outer| {
            choice((
                quantifier(choice((atom.clone(), outer.clone())), "∀|@", |v, e| {
                    Expr::Universal { iter: v, expr: e }
                }),
                quantifier(choice((atom.clone(), outer.clone())), "∃|/", |v, e| {
                    Expr::Existential { iter: v, expr: e }
                }),
                atom,
            ))
            .padded()
        });

        // expression with only constants
        let atom = choice((
            // special case for inequality == not equal
            constant_expr_atom(constant_expr())
                .then_ignore(regex("!=|≠").padded())
                .then(constant_expr_atom(constant_expr()))
                .map(|(a, b)| {
                    Expr::Not(Box::new(Expr::ConstantValue(ConstantExpr::Operator(
                        "=".to_string(),
                        Box::new(a),
                        Box::new(b),
                    ))))
                }),
            // normal  case
            constant_expr_atom(constant_expr())
                .then(constant_expr_operator())
                .then(constant_expr_atom(constant_expr()))
                .map(|((a, op), b)| {
                    Expr::ConstantValue(ConstantExpr::Operator(
                        op.to_string(),
                        Box::new(a),
                        Box::new(b),
                    ))
                }),
            atom,
        ));

        // not operator
        let atom = regex("¬|~|!").or_not().then(atom).map(|(n, v)| match n {
            Some(_) => Expr::Not(Box::new(v)),
            None => v,
        });

        // and, or, xor
        let atom = choice((
            infix_op_set(
                atom.clone(),
                vec![
                    (regex(r"∧|\*|&").boxed(), Expr::And as InfixOpMap),
                    (regex(r"∨|\+|\|").boxed(), Expr::Or),
                    (regex(r"⊕|(!∨)|(!\+)|(!\|)").boxed(), Expr::Xor),
                ],
                Expr::Invalid,
            ),
            atom,
        ))
        .padded();

        // conditional and biconditional
        let atom = choice((
            infix_op_set(
                atom.clone(),
                vec![
                    (regex("→|(->)").boxed(), Expr::Conditional as InfixOpMap),
                    (regex(r"↔|(<->)").boxed(), Expr::Biconditional),
                ],
                Expr::Invalid,
            ),
            atom,
        ))
        .padded();

        atom
    })
}
