use crate::types::{Point, Size, WindowId};
use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Encode, Decode)]
pub enum ClientMessage {
    WindowCreate { id: WindowId, size: Size },
    WindowDestroy { id: WindowId },
    BufferCommit { id: WindowId, data: Vec<u8> },
    WindowResize { id: WindowId, size: Size },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Encode, Decode)]
pub enum ServerMessage {
    KeyEvent {
        window_id: WindowId,
        scancode: u32,
        pressed: bool,
    },
    PointerMove {
        window_id: WindowId,
        position: Point,
    },
    PointerButton {
        window_id: WindowId,
        button: u32,
        pressed: bool,
    },
    WindowFocus {
        window_id: WindowId,
    },
    WindowClose {
        window_id: WindowId,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_message_roundtrip_bincode() {
        let msg = ClientMessage::WindowCreate {
            id: WindowId(42),
            size: Size {
                width: 1280,
                height: 720,
            },
        };
        let encoded = bincode::encode_to_vec(&msg, bincode::config::standard()).unwrap();
        let (decoded, _): (ClientMessage, _) =
            bincode::decode_from_slice(&encoded, bincode::config::standard()).unwrap();
        assert_eq!(msg, decoded);
    }

    #[test]
    fn server_message_roundtrip_bincode() {
        let msg = ServerMessage::KeyEvent {
            window_id: WindowId(1),
            scancode: 65,
            pressed: true,
        };
        let encoded = bincode::encode_to_vec(&msg, bincode::config::standard()).unwrap();
        let (decoded, _): (ServerMessage, _) =
            bincode::decode_from_slice(&encoded, bincode::config::standard()).unwrap();
        assert_eq!(msg, decoded);
    }

    #[test]
    fn buffer_commit_with_data() {
        let data = vec![0u8, 128, 255, 64];
        let msg = ClientMessage::BufferCommit {
            id: WindowId(7),
            data: data.clone(),
        };
        let encoded = bincode::encode_to_vec(&msg, bincode::config::standard()).unwrap();
        let (decoded, _): (ClientMessage, _) =
            bincode::decode_from_slice(&encoded, bincode::config::standard()).unwrap();
        assert_eq!(msg, decoded);
        if let ClientMessage::BufferCommit { data: d, .. } = decoded {
            assert_eq!(d, data);
        }
    }
}
