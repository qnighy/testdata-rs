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

/// Generates multiple test functions based on files.
///
/// ## Macro arguments
///
/// ### rebuild
///
/// Path to the test file, relative to the crate root.
///
/// This is necessary to rebuild the test binary
/// when test files are added or removed.
///
/// If omitted, the test binary may remain old even when test
/// files are added or removed.
///
/// ```rust,ignore
/// #[testdata::files(rebuild = "tests/example.rs")]
/// ```
///
/// ### root
///
/// Path to the directory where the macro starts walking,
/// relative to the crate root.
///
/// Defaults to `.`.
///
/// ```rust,ignore
/// #[testdata::files(root = "tests/fixtures")]
/// ```
///
/// ## Attributes on function arguments
///
/// ### glob
///
/// A glob to capture test files, relative to the `root` argument to the macro.
///
/// ```rust,ignore
/// fn f(
///     #[glob = "tests/fixtures/**/*-in.txt"]
///     input: &TestFile,
/// ) {}
/// ```
///
/// #### Wildcard in the glob
///
/// A glob must have exactly one wildcard part, which should be either `**/*` or `*`.
///
/// - `**/*` matches arbitrary part of paths (including ones without slashes)
/// - `*` matches a part of a single path segment. `*` does not match parts including `/`.
///
/// The wildcard is shared across different arguments.
/// For example, if there are an argument marked as `#[glob = "*-in.txt"]`
/// and one marked as `#[glob = "*-out.txt"]`, then the two `*`s in the glob match.
///
/// #### Comma in the glob
///
/// Additionally, you may provide more than one candidates in the glob by separating them
/// with commas (`,`).
/// In this case, each candidate must contain exactly one wildcard part.
///
/// If there are multiple candidates, they are treated like an overlay file system. That is,
///
/// - For reading, the first one existing will be picked.
/// - For writing, the first one, whether existing or not, will be picked.
///
/// ## Example
///
/// ```rust
/// use std::str;
/// use testdata::{assert_snapshot, TestFile};
///
/// #[testdata::files(rebuild = "tests/example.rs")]
/// #[test]
/// fn test_foo(
///     #[glob = "tests/fixtures/**/*-in.txt"] input: &TestFile,
///     #[glob = "tests/fixtures/**/*-out.txt"] output: &TestFile,
/// ) {
///     let s = input.raw_read();
///     let s = str::from_utf8(&s).unwrap();
///     let result = s.to_uppercase();
///     assert_snapshot!(result, snapshot = output);
/// }
/// ```
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
