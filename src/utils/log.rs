//! Utilities to help with logging.

/**
Equivalent of `::log::log!()` for the tracing crate.

Refs: <https://github.com/tokio-rs/tracing/issues/2730#issuecomment-1943022805>
*/
#[macro_export]
macro_rules! log {
    ($lvl:ident, $($arg:tt)+) => {
        match $lvl {
            ::tracing::Level::TRACE => ::tracing::trace!($($arg)+),
            ::tracing::Level::DEBUG => ::tracing::debug!($($arg)+),
            ::tracing::Level::INFO => ::tracing::info!($($arg)+),
            ::tracing::Level::WARN => ::tracing::warn!($($arg)+),
            ::tracing::Level::ERROR => ::tracing::error!($($arg)+),
        }
    };
}
