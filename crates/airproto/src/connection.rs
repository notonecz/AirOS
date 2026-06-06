use bincode::{Decode, Encode};
use std::path::Path;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;

pub struct AirProtoConn {
    stream: UnixStream,
}

impl AirProtoConn {
    pub async fn connect(path: &Path) -> std::io::Result<Self> {
        let stream = UnixStream::connect(path).await?;
        Ok(Self { stream })
    }

    pub fn from_stream(stream: UnixStream) -> Self {
        Self { stream }
    }

    /// Odešle zprávu jako length-prefixed frame.
    pub async fn send<T: Encode>(&mut self, msg: &T) -> std::io::Result<()> {
        let payload = bincode::encode_to_vec(msg, bincode::config::standard())
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;
        let len = u32::try_from(payload.len())
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "payload too large"))?;
        self.stream.write_all(&len.to_le_bytes()).await?;
        self.stream.write_all(&payload).await?;
        Ok(())
    }

    /// Přečte jednu zprávu z length-prefixed frame.
    pub async fn recv<T: Decode<()>>(&mut self) -> std::io::Result<T> {
        let mut len_buf = [0u8; 4];
        self.stream.read_exact(&mut len_buf).await?;
        let len = u32::from_le_bytes(len_buf) as usize;
        let mut payload = vec![0u8; len];
        self.stream.read_exact(&mut payload).await?;
        let (msg, _) = bincode::decode_from_slice(&payload, bincode::config::standard())
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;
        Ok(msg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::messages::{ClientMessage, ServerMessage};
    use crate::types::{Size, WindowId};
    use tempfile::TempDir;
    use tokio::net::UnixListener;

    #[tokio::test]
    async fn send_and_recv_client_message() {
        let dir = TempDir::new().unwrap();
        let socket_path = dir.path().join("test.sock");
        let listener = UnixListener::bind(&socket_path).unwrap();
        let path = socket_path.clone();
        let client_task = tokio::spawn(async move {
            let mut conn = AirProtoConn::connect(&path).await.unwrap();
            let msg = ClientMessage::WindowCreate {
                id: WindowId(1),
                size: Size {
                    width: 800,
                    height: 600,
                },
            };
            conn.send(&msg).await.unwrap();
        });
        let (server_stream, _) = listener.accept().await.unwrap();
        let mut server_conn = AirProtoConn::from_stream(server_stream);
        let received: ClientMessage = server_conn.recv().await.unwrap();
        client_task.await.unwrap();
        assert_eq!(
            received,
            ClientMessage::WindowCreate {
                id: WindowId(1),
                size: Size {
                    width: 800,
                    height: 600
                },
            }
        );
    }

    #[tokio::test]
    async fn send_and_recv_server_message() {
        let dir = TempDir::new().unwrap();
        let socket_path = dir.path().join("test2.sock");
        let listener = UnixListener::bind(&socket_path).unwrap();
        let server_task = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let mut conn = AirProtoConn::from_stream(stream);
            let msg = ServerMessage::WindowClose {
                window_id: WindowId(99),
            };
            conn.send(&msg).await.unwrap();
        });
        let mut client_conn = AirProtoConn::connect(&socket_path).await.unwrap();
        let received: ServerMessage = client_conn.recv().await.unwrap();
        server_task.await.unwrap();
        assert_eq!(
            received,
            ServerMessage::WindowClose {
                window_id: WindowId(99)
            }
        );
    }

    #[tokio::test]
    async fn multiple_messages_in_sequence() {
        let dir = TempDir::new().unwrap();
        let socket_path = dir.path().join("test3.sock");
        let listener = UnixListener::bind(&socket_path).unwrap();
        let path = socket_path.clone();
        let client_task = tokio::spawn(async move {
            let mut conn = AirProtoConn::connect(&path).await.unwrap();
            conn.send(&ClientMessage::WindowCreate {
                id: WindowId(1),
                size: Size {
                    width: 100,
                    height: 100,
                },
            })
            .await
            .unwrap();
            conn.send(&ClientMessage::WindowDestroy { id: WindowId(1) })
                .await
                .unwrap();
        });
        let (stream, _) = listener.accept().await.unwrap();
        let mut server_conn = AirProtoConn::from_stream(stream);
        let m1: ClientMessage = server_conn.recv().await.unwrap();
        let m2: ClientMessage = server_conn.recv().await.unwrap();
        client_task.await.unwrap();
        assert!(matches!(m1, ClientMessage::WindowCreate { .. }));
        assert!(matches!(m2, ClientMessage::WindowDestroy { .. }));
    }
}
