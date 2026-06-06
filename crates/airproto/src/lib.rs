// crates/airproto/src/lib.rs
pub mod connection;
pub mod messages;
pub mod types;

pub use connection::AirProtoConn;
pub use messages::{ClientMessage, ServerMessage};
pub use types::{Point, Size, WindowId};
