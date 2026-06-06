use crate::messages::BusMessage;
use std::path::Path;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;

pub struct AirBusConn {
    stream: UnixStream,
}

impl AirBusConn {
    pub async fn connect(path: &Path) -> std::io::Result<Self> {
        let stream = UnixStream::connect(path).await?;
        Ok(Self { stream })
    }

    pub fn from_stream(stream: UnixStream) -> Self {
        Self { stream }
    }

    pub async fn send(&mut self, msg: &BusMessage) -> std::io::Result<()> {
        let payload = rmp_serde::to_vec(msg)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;
        let len = u32::try_from(payload.len())
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "payload too large"))?;
        self.stream.write_all(&len.to_le_bytes()).await?;
        self.stream.write_all(&payload).await?;
        Ok(())
    }

    pub async fn recv(&mut self) -> std::io::Result<BusMessage> {
        let mut len_buf = [0u8; 4];
        self.stream.read_exact(&mut len_buf).await?;
        let len = u32::from_le_bytes(len_buf) as usize;
        let mut payload = vec![0u8; len];
        self.stream.read_exact(&mut payload).await?;
        rmp_serde::from_slice(&payload)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::messages::NotificationPayload;
    use tempfile::TempDir;
    use tokio::net::UnixListener;

    #[tokio::test]
    async fn send_notify_over_socket() {
        let dir = TempDir::new().unwrap();
        let socket_path = dir.path().join("airbus.sock");
        let listener = UnixListener::bind(&socket_path).unwrap();
        let path = socket_path.clone();
        let client_task = tokio::spawn(async move {
            let mut conn = AirBusConn::connect(&path).await.unwrap();
            conn.send(&BusMessage::Notify(NotificationPayload {
                title: "Hello".to_string(),
                body: "World".to_string(),
                icon: None,
            }))
            .await
            .unwrap();
        });
        let (stream, _) = listener.accept().await.unwrap();
        let mut server_conn = AirBusConn::from_stream(stream);
        let received = server_conn.recv().await.unwrap();
        client_task.await.unwrap();
        assert!(matches!(received, BusMessage::Notify(_)));
    }

    #[tokio::test]
    async fn send_dock_badge_over_socket() {
        let dir = TempDir::new().unwrap();
        let socket_path = dir.path().join("airbus2.sock");
        let listener = UnixListener::bind(&socket_path).unwrap();
        let path = socket_path.clone();
        let client_task = tokio::spawn(async move {
            let mut conn = AirBusConn::connect(&path).await.unwrap();
            conn.send(&BusMessage::SetDockBadge { count: 3 })
                .await
                .unwrap();
        });
        let (stream, _) = listener.accept().await.unwrap();
        let mut server_conn = AirBusConn::from_stream(stream);
        let received = server_conn.recv().await.unwrap();
        client_task.await.unwrap();
        assert_eq!(received, BusMessage::SetDockBadge { count: 3 });
    }
}
