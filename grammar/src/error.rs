use chumsky::{
    error::{Error, RichPattern, RichReason},
    label::LabelError,
    prelude::{Input, SimpleSpan},
    span::Span,
    util::MaybeRef,
};
use std::{
    cmp::{max, min},
    convert::From,
    fmt::{self, Debug, Display, Formatter},
    ops::Range,
};

pub struct YggError<'a, T, O = usize, S = SimpleSpan<O>> {
    spans: Vec<S>,
    start: O,
    end: O,
    reason: Box<RichReason<'a, T>>,
    context: Vec<(RichPattern<'a, T>, S)>,
}

impl<T, O, S> YggError<'_, T, O, S> {
    pub fn spans(&self) -> impl Iterator<Item = &S> {
        self.spans.iter()
    }

    pub fn reason(&self) -> &RichReason<'_, T> {
        &self.reason
    }

    pub fn start(&self) -> &O {
        &self.start
    }

    pub fn end(&self) -> &O {
        &self.end
    }
}

impl<T, O: PartialEq + Ord + Default, S: Into<Range<O>> + Clone> YggError<'_, T, O, S>
where
    Range<O>: From<S>,
{
    pub fn custom<M: ToString>(spans: Vec<S>, msg: M) -> Self {
        let start = spans.iter().fold(None, |a, v| match a {
            None => Some(Range::from(v.clone()).start),
            Some(a) => Some(min(a, Range::from(v.clone()).start)),
        });

        let end = spans.iter().fold(None, |a, v| match a {
            None => Some(Range::from(v.clone()).end),
            Some(a) => Some(min(a, Range::from(v.clone()).end)),
        });

        Self {
            spans,
            reason: Box::new(RichReason::Custom(msg.to_string())),
            context: Vec::new(),
            start: start.unwrap_or_default(),
            end: end.unwrap_or_default(),
        }
    }
}

impl<T: PartialEq + Display, O, S> std::fmt::Display for YggError<'_, T, O, S>
where
    Range<O>: From<S>,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.reason)
    }
}

impl<T: PartialEq + Debug, O, S> Debug for YggError<'_, T, O, S>
where
    Range<O>: From<S>,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.reason)
    }
}

impl<T, O: Clone> From<&YggError<'_, T, O>> for Range<O> {
    fn from(value: &YggError<T, O>) -> Self {
        (value.start().clone())..(value.end().clone())
    }
}

impl<'a, I: Input<'a>> Error<'a, I> for YggError<'a, I::Token, <I::Span as Span>::Offset, I::Span>
where
    I::Token: PartialEq,
    I::Span: Clone,
    Range<<I::Span as Span>::Offset>: From<I::Span>,
{
}

impl<'a, I: Input<'a>, L> LabelError<'a, I, L>
    for YggError<'a, I::Token, <I::Span as Span>::Offset, I::Span>
where
    I::Token: PartialEq,
    I::Span: Clone,
    Range<<I::Span as Span>::Offset>: From<I::Span>,
    L: Into<RichPattern<'a, I::Token>>,
{
    fn expected_found<E: IntoIterator<Item = L>>(
        expected: E,
        found: Option<MaybeRef<'a, I::Token>>,
        span: I::Span,
    ) -> Self {
        Self {
            spans: vec![span.clone()],
            reason: Box::new(RichReason::ExpectedFound {
                expected: expected.into_iter().map(|tok| tok.into()).collect(),
                found,
            }),
            context: Vec::new(),
            start: span.clone().start(),
            end: span.end(),
        }
    }
}
