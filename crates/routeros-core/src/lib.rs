//! Async client for the MikroTik RouterOS binary API.

pub mod client;
pub mod codec;
pub mod error;

pub use client::{build_command, Client, Subscription};
pub use codec::{Kind, Sentence};
pub use error::{Error, Result};
