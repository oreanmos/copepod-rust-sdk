pub mod api;
pub mod auth;
pub mod client;
pub mod error;
mod error_decode;
pub mod models;
pub mod query;
pub mod realtime;
pub mod scoped;

pub use client::{CopepodClient, CopepodClientBuilder};
pub use error::{
    CopepodError, CAPACITY_REACHED_CODE, RAFT_LEADER_UNAVAILABLE_CODE, STORAGE_QUOTA_EXCEEDED_CODE,
};
pub use models::*;
pub use scoped::*;
