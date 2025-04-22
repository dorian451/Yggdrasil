#![feature(iterator_try_collect)]
#![feature(if_let_guard)]
#![feature(let_chains)]

pub mod app;
pub mod error_template;
#[cfg(feature = "ssr")]
pub mod fileserv;

use std::{env, error::Error};
use tracing::level_filters::LevelFilter;
use tracing_error::ErrorLayer;
use tracing_panic::panic_hook;
use tracing_subscriber::{
    fmt::{format::FmtSpan, Layer},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter,
};

#[cfg(feature = "hydrate")]
use tracing_subscriber_wasm::MakeConsoleWriter;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use crate::app::*;

    console_error_panic_hook::set_once();
    init_tracing().unwrap();

    leptos::mount::hydrate_body(App);
}

#[cfg(feature = "ssr")]
fn new_layer<S>() -> Layer<S> {
    Layer::new()
}

#[cfg(all(feature = "hydrate", not(feature = "ssr")))]
fn new_layer<S>() -> Layer<
    S,
    tracing_subscriber::fmt::format::DefaultFields,
    tracing_subscriber::fmt::format::Format<tracing_subscriber::fmt::format::Full, ()>,
> {
    Layer::new().without_time()
}

pub fn init_tracing() -> Result<(), Box<dyn Error>> {
    let reg = tracing_subscriber::registry()
        .with(new_layer())
        .with(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .with_env_var("YGG_LOG")
                .from_env()?,
        )
        .with(ErrorLayer::default())
        .with(new_layer().with_span_events(
            if let Ok("1") = env::var("YGG_LOG_TRACE_SPAN").as_deref() {
                FmtSpan::NEW | FmtSpan::CLOSE
            } else {
                FmtSpan::NONE
            },
        ));

    #[cfg(all(feature = "hydrate", not(feature = "ssr")))]
    reg.with(new_layer().with_writer(MakeConsoleWriter::default()))
        .try_init()?;

    #[cfg(feature = "ssr")]
    reg.try_init()?;

    let prev_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        prev_hook(panic_info);
        panic_hook(panic_info);
    }));

    Ok(())
}
