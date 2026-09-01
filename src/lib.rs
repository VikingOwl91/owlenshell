pub mod domain;
pub mod instrumentation;
pub mod model;
pub mod persistence;
pub mod placement;
pub mod platform;
pub mod runtime;
pub mod ui {
    slint::include_modules!();
}
