pub mod api;
pub mod auth;
pub mod client;
pub mod error;
pub mod models;
pub mod query;
pub mod realtime;
pub mod scoped;

pub use client::{CopepodClient, CopepodClientBuilder};
pub use error::CopepodError;
pub use models::*;
pub use scoped::*;
