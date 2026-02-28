pub mod error;
pub mod models;
pub mod client;
pub mod auth;
pub mod query;
pub mod realtime;
pub mod api;

pub use client::{CopepodClient, CopepodClientBuilder};
pub use error::CopepodError;
pub use models::*;
