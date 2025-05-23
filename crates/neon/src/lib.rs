//! The [Neon][neon] crate provides bindings for writing [Node.js addons][addons]
//! (i.e., dynamically-loaded binary modules) with a safe and fast Rust API.
//!
//! ## Getting Started
//!
//! You can conveniently bootstrap a new Neon project with the Neon project
//! generator. You don't need to install anything special on your machine as
//! long as you have a [supported version of Node and Rust][supported] on
//! your system.
//!
//! To start a new project, open a terminal in the directory where you would
//! like to place the project, and run at the command prompt:
//!
//! ```text
//! % npm init neon my-project
//! ... answer the user prompts ...
//! ✨ Created Neon project `my-project`. Happy 🦀 hacking! ✨
//! ```
//!
//! where `my-project` can be any name you like for the project. This will
//! run the Neon project generator, prompting you with a few questions and
//! placing a simple but working Neon project in a subdirectory called
//! `my-project` (or whatever name you chose).
//!
//! You can then install and build the project by changing into the project
//! directory and running the standard Node installation command:
//!
//! ```text
//! % cd my-project
//! % npm install
//! % node
//! > require(".").hello()
//! 'hello node'
//! ```
//!
//! You can look in the project's generated `README.md` for more details on
//! the project structure.
//!
//! ## Example
//!
//! The generated `src/lib.rs` contains a function annotated with the
//! [`#[neon::main]`](main) attribute, marking it as the module's main entry
//! point to be executed when the module is loaded. This function can have
//! any name but is conventionally called `main`:
//!
//! ```no_run
//! # mod example {
//! # use neon::prelude::*;
//! # fn hello(mut cx: FunctionContext) -> JsResult<JsString> {
//! #    Ok(cx.string("hello node"))
//! # }
//! #[neon::main]
//! fn main(mut cx: ModuleContext) -> NeonResult<()> {
//!     cx.export_function("hello", hello)?;
//!     Ok(())
//! }
//! # }
//! ```
//!
//! The example code generated by `npm init neon` exports a single
//! function via [`ModuleContext::export_function`](context::ModuleContext::export_function).
//! The `hello` function is defined just above `main` in `src/lib.rs`:
//!
//! ```
//! # use neon::prelude::*;
//! #
//! fn hello(mut cx: FunctionContext) -> JsResult<JsString> {
//!     Ok(cx.string("hello node"))
//! }
//! ```
//!
//! The `hello` function takes a [`FunctionContext`](context::FunctionContext) and
//! returns a JavaScript string. Because all Neon functions can potentially throw a
//! JavaScript exception, the return type is wrapped in a [`JsResult`](result::JsResult).
//!
//! [neon]: https://www.neon-bindings.com/
//! [addons]: https://nodejs.org/api/addons.html
//! [supported]: https://github.com/neon-bindings/neon#platform-support
#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod context;
pub mod event;
pub mod handle;
mod macros;
pub mod meta;
pub mod object;
pub mod prelude;
pub mod reflect;
pub mod result;
#[cfg(not(feature = "sys"))]
mod sys;
#[cfg_attr(docsrs, doc(cfg(feature = "napi-6")))]
#[cfg(feature = "napi-6")]
pub mod thread;
// To use the #[aquamarine] attribute on the top-level neon::types module docs, we have to
// use this hack so we can keep the module docs in a separate file.
// See: https://github.com/mersinvald/aquamarine/issues/5#issuecomment-1168816499
mod types_docs;
mod types_impl;

#[cfg(feature = "sys")]
#[cfg_attr(docsrs, doc(cfg(feature = "sys")))]
pub mod sys;

#[cfg(all(feature = "napi-6", feature = "futures"))]
#[cfg_attr(docsrs, doc(cfg(all(feature = "napi-6", feature = "futures"))))]
pub use executor::set_global_executor;
pub use types_docs::exports as types;

#[doc(hidden)]
pub mod macro_internal;

pub use crate::macros::*;

use crate::{context::ModuleContext, handle::Handle, result::NeonResult, types::JsValue};

#[cfg(feature = "napi-6")]
mod lifecycle;

#[cfg(all(feature = "napi-6", feature = "futures"))]
mod executor;

#[cfg(feature = "napi-8")]
static MODULE_TAG: once_cell::sync::Lazy<crate::sys::TypeTag> = once_cell::sync::Lazy::new(|| {
    let mut lower = [0; std::mem::size_of::<u64>()];

    // Generating a random module tag at runtime allows Neon builds to be reproducible. A few
    //  alternatives considered:
    // * Generating a random value at build time; this reduces runtime dependencies but, breaks
    //   reproducible builds
    // * A static random value; this solves the previous issues, but does not protect against ABI
    //   differences across Neon and Rust versions
    // * Calculating a variable from the environment (e.g. Rust version); this theoretically works
    //   but, is complicated and error prone. This could be a future optimization.
    getrandom::getrandom(&mut lower).expect("Failed to generate a Neon module type tag");

    // We only use 64-bits of the available 128-bits. The rest is reserved for future versioning and
    // expansion of implementation.
    let lower = u64::from_ne_bytes(lower);

    // Note: `upper` must be non-zero or `napi_check_object_type_tag` will always return false
    // https://github.com/nodejs/node/blob/5fad0b93667ffc6e4def52996b9529ac99b26319/src/js_native_api_v8.cc#L2455
    crate::sys::TypeTag { lower, upper: 1 }
});

/// Values exported with [`neon::export`](export)
pub struct Exports(());

impl Exports {
    /// Export all values exported with [`neon::export`](export)
    ///
    /// ```
    /// # fn main() {
    /// # use neon::prelude::*;
    /// #[neon::main]
    /// fn main(mut cx: ModuleContext) -> NeonResult<()> {
    ///     neon::registered().export(&mut cx)?;
    ///     Ok(())
    /// }
    /// # }
    /// ```
    ///
    /// For more control, iterate over exports.
    ///
    /// ```
    /// # fn main() {
    /// # use neon::prelude::*;
    /// #[neon::main]
    /// fn main(mut cx: ModuleContext) -> NeonResult<()> {
    ///     for create in neon::registered() {
    ///         let (name, value) = create(&mut cx)?;
    ///
    ///         cx.export_value(name, value)?;
    ///     }
    ///
    ///     Ok(())
    /// }
    /// # }
    /// ```
    pub fn export(self, cx: &mut ModuleContext) -> NeonResult<()> {
        for create in self {
            let (name, value) = create(cx)?;

            cx.export_value(name, value)?;
        }

        Ok(())
    }
}

impl IntoIterator for Exports {
    type Item = <<Self as IntoIterator>::IntoIter as IntoIterator>::Item;
    type IntoIter = std::slice::Iter<
        'static,
        for<'cx> fn(&mut ModuleContext<'cx>) -> NeonResult<(&'static str, Handle<'cx, JsValue>)>,
    >;

    fn into_iter(self) -> Self::IntoIter {
        crate::macro_internal::EXPORTS.into_iter()
    }
}

/// Access values exported with [`neon::export`](export)
pub fn registered() -> Exports {
    Exports(())
}

#[test]
fn feature_matrix() {
    use std::{env, process::Command};

    // N.B.: Only versions that are used are included in order to keep the set
    // of permutations as small as possible.
    const NODE_API_VERSIONS: &[&str] = &["napi-1", "napi-4", "napi-5", "napi-6", "napi-8"];

    const FEATURES: &[&str] = &["external-buffers", "futures", "serde", "tokio", "tokio-rt"];

    let cargo = env::var_os("CARGO").unwrap_or_else(|| "cargo".into());

    for features in itertools::Itertools::powerset(FEATURES.iter()) {
        for version in NODE_API_VERSIONS.iter().map(|f| f.to_string()) {
            let features = features.iter().fold(version, |f, s| f + "," + s);
            let status = Command::new(&cargo)
                .args(["check", "-p", "neon", "--no-default-features", "--features"])
                .arg(features)
                .spawn()
                .unwrap()
                .wait()
                .unwrap();

            assert!(status.success());
        }
    }
}
