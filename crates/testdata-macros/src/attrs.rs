use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{parse, Attribute, FnArg, Lit, Meta, NestedMeta, Token};

#[derive(Debug, Clone)]
pub(crate) struct MacroArgs {
    pub(crate) rebuild: Option<String>,
    pub(crate) root: Option<String>,
}

impl MacroArgs {
    pub(crate) fn parse(raw: TokenStream) -> Result<Self, syn::Error> {
        let meta = parse::Parser::parse2(Punctuated::parse_terminated, raw)?;
        Self::parse_meta(&meta)
    }

    pub(crate) fn parse_meta(meta: &Punctuated<NestedMeta, Token![,]>) -> Result<Self, syn::Error> {
        let mut rebuild = None;
        let mut root = None;
        for arg in meta {
            if let NestedMeta::Meta(arg) = arg {
                if arg.path().is_ident("root") {
                    if root.is_some() {
                        return Err(syn::Error::new(arg.path().span(), "duplicate argument"));
                    }
                    if let Meta::NameValue(arg) = arg {
                        if let Lit::Str(lit) = &arg.lit {
                            root = Some(lit.value());
                            continue;
                        } else {
                            return Err(syn::Error::new(arg.lit.span(), "invalid argument value"));
                        }
                    } else {
                        return Err(syn::Error::new(arg.span(), "invalid argument value"));
                    }
                } else if arg.path().is_ident("rebuild") {
                    if rebuild.is_some() {
                        return Err(syn::Error::new(arg.path().span(), "duplicate argument"));
                    }
                    if let Meta::NameValue(arg) = arg {
                        if let Lit::Str(lit) = &arg.lit {
                            rebuild = Some(lit.value());
                            continue;
                        } else {
                            return Err(syn::Error::new(arg.lit.span(), "invalid argument value"));
                        }
                    } else {
                        return Err(syn::Error::new(arg.span(), "invalid argument value"));
                    }
                } else {
                    return Err(syn::Error::new(
                        arg.path().span(),
                        format_args!("unknown argument: {}", arg.path().to_token_stream()),
                    ));
                }
            } else {
                return Err(syn::Error::new(arg.span(), "invalid argument"));
            };
        }
        Ok(MacroArgs { rebuild, root })
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ArgAttrs {
    pub(crate) glob: String,
}

impl ArgAttrs {
    pub(crate) fn parse(arg: &FnArg) -> Result<Self, syn::Error> {
        let attrs = match arg {
            FnArg::Receiver(arg) => &arg.attrs,
            FnArg::Typed(arg) => &arg.attrs,
        };
        Self::parse_attrs(attrs, arg.span())
    }
    pub(crate) fn parse_attrs(attrs: &[Attribute], span: Span) -> Result<Self, syn::Error> {
        let mut glob = None;
        for attr in attrs {
            let meta = if let Ok(meta) = attr.parse_meta() {
                meta
            } else {
                continue;
            };
            if meta.path().is_ident("glob") {
                if glob.is_some() {
                    return Err(syn::Error::new(attr.span(), "Duplicate #[glob] attribute"));
                }
                let meta = if let Meta::NameValue(meta) = &meta {
                    meta
                } else {
                    return Err(syn::Error::new(meta.span(), "Expected #[glob = ...]"));
                };
                let lit = if let Lit::Str(lit) = &meta.lit {
                    lit
                } else {
                    return Err(syn::Error::new(
                        meta.lit.span(),
                        "Expected a string literal",
                    ));
                };
                glob = Some(lit.value());
            }
        }
        let glob = glob.ok_or_else(|| syn::Error::new(span, "Missing argument: #[glob = ...]"))?;
        Ok(Self { glob })
    }
}

#[cfg(test)]
mod tests {
    use quote::quote;
    use syn::parse_quote;

    use super::*;

    #[test]
    fn test_parse_macro_args_empty() {
        let args = quote! {};
        let args = MacroArgs::parse(args).unwrap();
        assert_eq!(args.rebuild, None);
        assert_eq!(args.root, None);
    }

    #[test]
    fn test_parse_macro_args_root() {
        let args = quote! {root = "."};
        let args = MacroArgs::parse(args).unwrap();
        assert_eq!(args.rebuild, None);
        assert_eq!(args.root, Some(".".to_owned()));
    }

    #[test]
    fn test_parse_macro_args_rebuild() {
        let args = quote! {rebuild = "tests/test.rs"};
        let args = MacroArgs::parse(args).unwrap();
        assert_eq!(args.rebuild, Some("tests/test.rs".to_owned()));
        assert_eq!(args.root, None);
    }

    #[test]
    fn test_parse_macro_args_unknown_arg() {
        let args = quote! {foo = 42};
        let e = MacroArgs::parse(args).unwrap_err();
        assert_eq!(e.to_string(), "unknown argument: foo");
    }

    #[test]
    fn test_parse_macro_args_duplicate_arg() {
        {
            let args = quote! {root = ".", root = "tests"};
            let e = MacroArgs::parse(args).unwrap_err();
            assert_eq!(e.to_string(), "duplicate argument");
        }
        {
            let args = quote! {rebuild = "tests/foo.rs", rebuild = "tests/bar.rs"};
            let e = MacroArgs::parse(args).unwrap_err();
            assert_eq!(e.to_string(), "duplicate argument");
        }
    }

    #[test]
    fn test_parse_macro_args_invalid_arg() {
        {
            let args = quote! {"foo"};
            let e = MacroArgs::parse(args).unwrap_err();
            assert_eq!(e.to_string(), "invalid argument");
        }
        {
            let args = quote! {root = 42};
            let e = MacroArgs::parse(args).unwrap_err();
            assert_eq!(e.to_string(), "invalid argument value");
        }
        {
            let args = quote! {root(".")};
            let e = MacroArgs::parse(args).unwrap_err();
            assert_eq!(e.to_string(), "invalid argument value");
        }
        {
            let args = quote! {rebuild = 42};
            let e = MacroArgs::parse(args).unwrap_err();
            assert_eq!(e.to_string(), "invalid argument value");
        }
        {
            let args = quote! {rebuild(".")};
            let e = MacroArgs::parse(args).unwrap_err();
            assert_eq!(e.to_string(), "invalid argument value");
        }
    }

    #[test]
    fn test_parse_arg_attrs() {
        let item = parse_quote! {
            #[glob = "tests/fixtures/**/*-in.txt"]
            #[other_attr.invalid]
            x: PathBuf
        };
        let attrs = ArgAttrs::parse(&item).unwrap();
        assert_eq!(attrs.glob, "tests/fixtures/**/*-in.txt");
    }

    #[test]
    fn test_parse_arg_attrs_self() {
        let item = parse_quote! {
            #[glob = "tests/fixtures/**/*-in.txt"]
            #[other_attr.invalid]
            &self
        };
        let attrs = ArgAttrs::parse(&item).unwrap();
        assert_eq!(attrs.glob, "tests/fixtures/**/*-in.txt");
    }

    #[test]
    fn test_parse_arg_attrs_empty() {
        let item = parse_quote! {
            x: PathBuf
        };
        let e = ArgAttrs::parse(&item).unwrap_err();
        assert_eq!(e.to_string(), "Missing argument: #[glob = ...]");
    }

    #[test]
    fn test_parse_arg_attrs_duplicate() {
        let item = parse_quote! {
            #[glob = "tests/fixtures/**/*-in.txt"]
            #[glob = "tests/fixtures/**/*-out.txt"]
            x: PathBuf
        };
        let e = ArgAttrs::parse(&item).unwrap_err();
        assert_eq!(e.to_string(), "Duplicate #[glob] attribute");
    }

    #[test]
    fn test_parse_arg_attrs_invalid_format() {
        let item = parse_quote! {
            #[glob("tests/fixtures/**/*-in.txt")]
            x: PathBuf
        };
        let e = ArgAttrs::parse(&item).unwrap_err();
        assert_eq!(e.to_string(), "Expected #[glob = ...]");
    }

    #[test]
    fn test_parse_arg_attrs_invalid_value() {
        let item = parse_quote! {
            #[glob = 42]
            x: PathBuf
        };
        let e = ArgAttrs::parse(&item).unwrap_err();
        assert_eq!(e.to_string(), "Expected a string literal");
    }
}
