use std::collections::HashMap;

use proc_macro2::{Ident, Literal, Span, TokenStream};
use quote::quote;
use syn::{punctuated::Punctuated, FnArg, ItemFn, Token};
use testdata_glob::GlobSpec;

use crate::tree::{StemFn, StemTree};

pub(crate) fn generate(spec: &GlobSpec, item: &ItemFn, stems: &[String]) -> TokenStream {
    let spec_def = generate_glob_spec(spec);

    let function_name = &item.sig.ident;
    let tree = StemTree::build(&stems);

    let tree_tokens = generate_tree(&tree, 0, &item.sig.inputs, function_name);

    let base_function = {
        let mut base_function = item.clone();
        // Remove #[test] from the function attributes
        base_function.attrs.retain(|attr| {
            if let Ok(meta) = attr.parse_meta() {
                !meta.path().is_ident("test")
            } else {
                true
            }
        });
        for arg in &mut base_function.sig.inputs {
            let attrs = match arg {
                FnArg::Receiver(arg) => &mut arg.attrs,
                FnArg::Typed(arg) => &mut arg.attrs,
            };
            // Remove #[glob = "..."] from the parameter attributes
            attrs.retain(|attr| {
                if let Ok(meta) = attr.parse_meta() {
                    !meta.path().is_ident("glob")
                } else {
                    true
                }
            });
        }
        base_function
    };
    let fallback_fn = generate_fallback_fn(stems, &item.sig.inputs, function_name);

    quote! {
        #[cfg(test)]
        #base_function

        #[cfg(test)]
        mod #function_name {
            #spec_def

            #tree_tokens

            #fallback_fn
        }
    }
}

fn generate_glob_spec(spec: &GlobSpec) -> TokenStream {
    let rt = get_rt();
    let args = spec
        .args
        .iter()
        .map(|arg| {
            let path = &arg.path;
            quote! {
                .arg(#rt::ArgSpec::new(#path))
            }
        })
        .collect::<Vec<_>>();
    quote! {
        const __GLOB_SPEC: #rt::Lazy<#rt::GlobSpec> = #rt::Lazy::new(|| {
            #rt::GlobSpec::new()
                #(#args)*
        });
    }
}

fn generate_tree(
    tree: &StemTree,
    depth: usize,
    args: &Punctuated<FnArg, Token![,]>,
    base_function_name: &Ident,
) -> TokenStream {
    let fns = sorted_iter(&tree.fns)
        .map(|(name, def)| generate_fn(name, def, depth, args, base_function_name))
        .collect::<Vec<_>>();

    let mods = sorted_iter(&tree.mods)
        .map(|(name, def)| {
            let name = Ident::new(name, Span::call_site());
            let sub = generate_tree(def, depth + 1, args, base_function_name);
            quote! {
                mod #name {
                    #sub
                }
            }
        })
        .collect::<Vec<_>>();

    quote! {
        #(#fns)*
        #(#mods)*
    }
}

fn generate_fn(
    name: &str,
    def: &StemFn,
    depth: usize,
    args: &Punctuated<FnArg, Token![,]>,
    base_function_name: &Ident,
) -> TokenStream {
    let self_ref = up(depth);
    let super_ref = up(depth + 1);
    let name = Ident::new(name, Span::call_site());
    let stem = &def.stem;
    let arg_forwards = (0..args.len())
        .map(|i| {
            let i = Literal::usize_unsuffixed(i);
            quote! {
                &paths[#i]
            }
        })
        .collect::<Vec<_>>();
    quote! {
        #[test]
        fn #name() {
            if let Some(paths) = #self_ref::__GLOB_SPEC.expand(std::path::Path::new("."), #stem) {
                #super_ref::#base_function_name(#(#arg_forwards),*);
            }
        }
    }
}

fn generate_fallback_fn(
    stems: &[String],
    args: &Punctuated<FnArg, Token![,]>,
    base_function_name: &Ident,
) -> TokenStream {
    let stems_literal = stems
        .iter()
        .map(|stem| quote! { #stem.to_owned() })
        .collect::<Vec<_>>();
    let stems_literal = quote! {
        vec![#(#stems_literal),*]
    };
    let arg_forwards = (0..args.len())
        .map(|i| {
            let i = Literal::usize_unsuffixed(i);
            quote! {
                &paths[#i]
            }
        })
        .collect::<Vec<_>>();
    quote! {
        #[test]
        fn __others() {
            let known_stems = #stems_literal;
            let (extra_stems, missing_stems) = self::__GLOB_SPEC
                .glob_diff(std::path::Path::new("."), &known_stems)
                .unwrap();
            for stem in &extra_stems {
                if known_stems.contains(stem) {
                    continue;
                }
                let paths = self::__GLOB_SPEC
                    .expand(std::path::Path::new("."), stem)
                    .unwrap();
                super::#base_function_name(#(#arg_forwards),*);
            }
            if !extra_stems.is_empty() || !missing_stems.is_empty() {
                // TODO: recompile
            }
        }
    }
}

fn sorted_iter<'a, K, V, S>(h: &'a HashMap<K, V, S>) -> impl Iterator<Item = (&'a K, &'a V)> + 'a
where
    K: Ord + std::hash::Hash,
    S: std::hash::BuildHasher,
{
    let mut keys = h.keys().collect::<Vec<_>>();
    keys.sort();
    keys.into_iter().map(move |k| (k, &h[k]))
}

fn up(depth: usize) -> TokenStream {
    if depth == 0 {
        quote! { self }
    } else {
        let mut tokens = Vec::with_capacity(depth * 2 - 1);
        for i in 0..depth {
            if i == 0 {
                tokens.extend(quote! { super });
            } else {
                tokens.extend(quote! { ::super });
            }
        }
        tokens.into_iter().collect()
    }
}

fn get_rt() -> TokenStream {
    quote! {
        testdata::__rt
    }
}

#[cfg(test)]
mod tests {
    use big_s::S;
    use syn::parse_quote;
    use testdata_glob::ArgSpec;

    use crate::assert_ts_eq;

    use super::*;

    #[test]
    fn test_generate() {
        let item = parse_quote! {
            #[test]
            fn test_foo(
                #[glob = "tests/fixtures/**/*-in.txt"]
                input: PathBuf,
                #[glob = "tests/fixtures/**/*-out.txt"]
                output: PathBuf,
            ) {
                foo();
            }
        };
        let spec = GlobSpec::new()
            .arg(ArgSpec::new("tests/fixtures/**/*-in.txt"))
            .arg(ArgSpec::new("tests/fixtures/**/*-out.txt"));
        let tokens = generate(
            &spec,
            &item,
            &[
                S("bar"),
                S("foo"),
                S("foo/bar-baz"),
                S("foo/bar/01_todo"),
                S("foo/bar/baz"),
                S("foo/bar_baz"),
            ],
        );
        assert_ts_eq!(
            tokens,
            quote! {
                #[cfg(test)]
                fn test_foo(input: PathBuf, output: PathBuf,) {
                    foo();
                }
                #[cfg(test)]
                mod test_foo {
                    const __GLOB_SPEC: testdata::__rt::Lazy<testdata::__rt::GlobSpec> =
                        testdata::__rt::Lazy::new(|| {
                            testdata::__rt::GlobSpec::new()
                                .arg(testdata::__rt::ArgSpec::new("tests/fixtures/**/*-in.txt"))
                                .arg(testdata::__rt::ArgSpec::new("tests/fixtures/**/*-out.txt"))
                        });
                    #[test]
                    fn bar() {
                        if let Some(paths) = self::__GLOB_SPEC.expand(std::path::Path::new("."), "bar") {
                            super::test_foo(&paths[0], &paths[1]);
                        }
                    }
                    #[test]
                    fn foo() {
                        if let Some(paths) = self::__GLOB_SPEC.expand(std::path::Path::new("."), "foo") {
                            super::test_foo(&paths[0], &paths[1]);
                        }
                    }
                    mod foo {
                        #[test]
                        fn bar_baz() {
                            if let Some(paths) = super::__GLOB_SPEC.expand(std::path::Path::new("."), "foo/bar-baz") {
                                super::super::test_foo(&paths[0], &paths[1]);
                            }
                        }
                        #[test]
                        fn bar_baz_1() {
                            if let Some(paths) = super::__GLOB_SPEC.expand(std::path::Path::new("."), "foo/bar_baz") {
                                super::super::test_foo(&paths[0], &paths[1]);
                            }
                        }
                        mod bar {
                            #[test]
                            fn _01_todo() {
                                if let Some(paths) = super::super::__GLOB_SPEC.expand(std::path::Path::new("."), "foo/bar/01_todo") {
                                    super::super::super::test_foo(&paths[0], &paths[1]);
                                }
                            }
                            #[test]
                            fn baz() {
                                if let Some(paths) = super::super::__GLOB_SPEC.expand(std::path::Path::new("."), "foo/bar/baz") {
                                    super::super::super::test_foo(&paths[0], &paths[1]);
                                }
                            }
                        }
                    }
                    #[test]
                    fn __others() {
                        let known_stems = vec![
                            "bar".to_owned(),
                            "foo".to_owned(),
                            "foo/bar-baz".to_owned(),
                            "foo/bar/01_todo".to_owned(),
                            "foo/bar/baz".to_owned(),
                            "foo/bar_baz".to_owned()
                        ];
                        let (extra_stems, missing_stems) = self::__GLOB_SPEC
                            .glob_diff(std::path::Path::new("."), &known_stems)
                            .unwrap();
                        for stem in &extra_stems {
                            if known_stems.contains(stem) {
                                continue;
                            }
                            let paths = self::__GLOB_SPEC
                                .expand(std::path::Path::new("."), stem)
                                .unwrap();
                            super::test_foo(&paths[0], &paths[1]);
                        }
                        if !extra_stems.is_empty() || !missing_stems.is_empty() {}
                    }
                }
            }
        );
    }
}
