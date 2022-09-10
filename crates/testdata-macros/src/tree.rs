use std::{
    borrow::Cow,
    collections::{hash_map::Entry, HashMap},
};

use crate::sanitization::sanitize_component;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct StemTree {
    pub(crate) fns: HashMap<String, StemFn>,
    pub(crate) mods: HashMap<String, StemTree>,
}

impl StemTree {
    pub(crate) fn build(stems: &[String]) -> Self {
        let mut tree = Self::default();
        for stem in stems {
            tree.add(stem);
        }
        tree
    }

    pub(crate) fn add(&mut self, stem: &str) {
        let mut current = self;
        let parts = stem.split('/').collect::<Vec<_>>();
        for &part in &parts[..parts.len() - 1] {
            let part = sanitize_component(part);
            current = current.mods.entry(part).or_default();
        }
        {
            let part = parts[parts.len() - 1];
            for n in 0.. {
                let part = if n == 0 {
                    Cow::Borrowed(part)
                } else {
                    Cow::Owned(format!("{}_{}", part, n))
                };
                let part = sanitize_component(&part);
                if let Entry::Vacant(entry) = current.fns.entry(part) {
                    entry.insert(StemFn {
                        stem: stem.to_owned(),
                    });
                    break;
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct StemFn {
    pub(crate) stem: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use big_s::S;
    use maplit::hashmap;

    #[test]
    fn test_build() {
        let tree = StemTree::build(&[
            S("bar"),
            S("foo"),
            S("foo/bar-baz"),
            S("foo/bar/01_todo"),
            S("foo/bar/baz"),
            S("foo/bar_baz"),
        ]);
        assert_eq!(
            tree,
            StemTree {
                fns: hashmap! {
                    S("bar") => StemFn {
                        stem: S("bar"),
                    },
                    S("foo") => StemFn {
                        stem: S("foo"),
                    },
                },
                mods: hashmap! {
                    S("foo") => StemTree {
                        fns: hashmap! {
                            S("bar_baz") => StemFn {
                                stem: S("foo/bar-baz")
                            },
                            S("bar_baz_1") => StemFn {
                                stem: S("foo/bar_baz")
                            },
                        },
                        mods: hashmap! {
                            S("bar") => StemTree {
                                fns: hashmap! {
                                    S("_01_todo") => StemFn {
                                        stem: S("foo/bar/01_todo"),
                                    },
                                    S("baz") => StemFn {
                                        stem: S("foo/bar/baz"),
                                    },
                                },
                                mods: hashmap! {},
                            }
                        },
                    }
                },
            }
        );
    }
}
