// crates/airproto/src/lib.rs
pub mod types;
pub mod messages;
pub mod connection;

pub use types::{WindowId, Size, Point};
pub use messages::{ClientMessage, ServerMessage};
pub use connection::AirProtoConn;
