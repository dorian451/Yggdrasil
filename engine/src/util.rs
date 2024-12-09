use std::{
    collections::HashSet,
    hash::{DefaultHasher, Hash, Hasher},
    ops::Deref,
};

use yggdrasil_grammar::Expr;

pub fn expr_list_starts_with<'a, L: Iterator<Item = &'a Expr>>(
    mut list: L,
    set: &HashSet<Expr>,
) -> bool {
    let mut found = 0;
    while let (Some(v), false) = (list.next(), found >= set.len()) {
        if set.contains(v) {
            found += 1;
        } else {
            return false;
        }
    }
    found >= set.len()
}

pub fn expr_maybe_list_starts_with(
    mut list: impl Iterator<Item = impl Deref<Target = Option<Expr>>>,
    set: &HashSet<Expr>,
) -> bool {
    let mut found = 0;
    let mut matched = HashSet::new();

    while let (Some(v), false) = (list.next(), found >= set.len()) {
        let h = {
            let mut h = DefaultHasher::new();
            v.hash(&mut h);
            h.finish()
        };

        match v.deref() {
            Some(v) if set.contains(v) && !matched.contains(&h) => {
                found += 1;
                matched.insert(h);
            }
            _ => return false,
        }
    }
    found >= set.len()
}
