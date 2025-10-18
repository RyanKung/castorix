//! REST API module for Castorix
//!
//! Provides a RESTful HTTP API server for interacting with Farcaster protocol

pub mod handlers;
pub mod routes;
pub mod server;
pub mod types;

pub use server::ApiServer;
pub use types::{ApiError, ApiResponse};

