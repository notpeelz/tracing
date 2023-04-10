use std::{
    cell::RefCell,
    mem::MaybeUninit,
    sync::atomic::{AtomicBool, Ordering},
};

use proc_macro2::TokenStream;
use quote::{ToTokens, TokenStreamExt};
use syn::Path;

use crate::attr::InstrumentArgs;

#[derive(Clone)]
pub(crate) struct TracingPathPrefix(Path);

impl ToTokens for TracingPathPrefix {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(self.0.to_token_stream());
    }
}

thread_local! {
  static INITIALIZED: AtomicBool = AtomicBool::new(false);
  static PATH_PREFIX: RefCell<MaybeUninit<TracingPathPrefix>> = RefCell::new(MaybeUninit::uninit());
}

#[cfg(not(feature = "electronpipe"))]
pub(crate) fn get_path_prefix() -> TracingPathPrefix {
    TracingPathPrefix(syn::parse_quote!(::tracing))
}

#[cfg(not(feature = "electronpipe"))]
pub(crate) fn with_args<R>(args: InstrumentArgs, f: impl FnOnce() -> R) -> R {
    f()
}

#[cfg(feature = "electronpipe")]
pub(crate) fn get_path_prefix() -> TracingPathPrefix {
    if !INITIALIZED.with(|x| x.load(Ordering::Relaxed)) {
        panic!("path prefix isn't set");
    }
    PATH_PREFIX.with(|x| unsafe { x.borrow().assume_init_ref().clone() })
}

#[cfg(feature = "electronpipe")]
pub(crate) fn with_args<R>(args: InstrumentArgs, f: impl FnOnce() -> R) -> R {
    if INITIALIZED.with(|x| x.fetch_or(true, Ordering::Relaxed)) {
        panic!("path prefix was already set");
    }
    PATH_PREFIX.with(|x| {
        x.try_borrow_mut()
            .unwrap()
            .write(TracingPathPrefix(args.path_prefix.unwrap()));
    });
    let r = f();
    INITIALIZED.with(|x| x.store(false, Ordering::Relaxed));
    r
}
