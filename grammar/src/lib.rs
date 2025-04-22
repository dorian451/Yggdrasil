pub mod expr;
mod parser;

pub use chumsky::Parser;

use chumsky::{
    cache::{Cache, Cached},
    Boxed,
};
use expr::Expr;
use parser::{parser, Extras, Input};
use std::cell::LazyCell;

pub type YggdrasilGrammarParserType<'a, 'b, I = Input<'a>> = Boxed<'a, 'b, I, Expr, Extras<'a>>;

#[derive(Default)]
pub struct YggdrasilGrammarParserCache;

impl Cached for YggdrasilGrammarParserCache {
    type Parser<'src> = YggdrasilGrammarParserType<'src, 'src>;

    fn make_parser<'src>(self) -> Self::Parser<'src> {
        parser().boxed()
    }
}

thread_local! {
    pub static PARSER: LazyCell<Cache<YggdrasilGrammarParserCache>> = LazyCell::new(Cache::default);
}
