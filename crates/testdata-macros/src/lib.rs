mod attrs;
mod codegen;
mod sanitization;
#[cfg(test)]
mod testing;
mod tree;

use std::env;
use std::path::PathBuf;

use proc_macro2::TokenStream;
use syn::spanned::Spanned;
use syn::{parse2, Item};
use testdata_rt::GlobSpec;

use crate::attrs::{ArgAttrs, MacroArgs};
use crate::codegen::generate;

#[proc_macro_attribute]
pub fn files(
    raw_args: proc_macro::TokenStream,
    raw_item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    match files2(raw_args.into(), raw_item.into()) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn files2(raw_args: TokenStream, raw_item: TokenStream) -> Result<TokenStream, syn::Error> {
    let span = raw_args.span();
    let args = MacroArgs::parse(raw_args)?;

    let item: Item = parse2(raw_item)?;
    let item = if let Item::Fn(item) = item {
        item
    } else {
        return Err(syn::Error::new(
            item.span(),
            "expected function after #[testdata]",
        ));
    };

    let mut args_attrs = Vec::new();
    for arg in &item.sig.inputs {
        let attrs = ArgAttrs::parse(arg)?;
        args_attrs.push(attrs);
    }

    let mut spec = GlobSpec::new();
    if let Some(root) = &args.root {
        spec.root = root.into();
    }
    for attrs in &args_attrs {
        spec.args
            .push(testdata_rt::ArgSpec::parse(&attrs.glob).map_err(|e| syn::Error::new(span, e))?);
    }

    let cwd = env::var_os("CARGO_MANIFEST_DIR")
        .ok_or_else(|| syn::Error::new(span, "Missing CARGO_MANIFEST_DIR"))?;
    let cwd = PathBuf::from(cwd);
    let stems = spec.glob_from(&cwd).map_err(|e| syn::Error::new(span, e))?;

    Ok(generate(&spec, &args, &item, &stems))
}
