//! UI front-ends for moldx.
//!
//! Two interfaces are provided:
//! * [`tui`] — interactive terminal UI built with [ratatui](https://ratatui.rs)
//! * [`web`] — browser-based UI served by [axum](https://docs.rs/axum)
pub mod tui;
pub mod web;
