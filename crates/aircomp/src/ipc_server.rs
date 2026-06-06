use crate::scene::WindowScene;
use airproto::{AirProtoConn, ClientMessage};
use std::path::Path;
use std::sync::{Arc, Mutex};
use tokio::net::{UnixListener, UnixStream};

/// Zpracovává zprávy od jednoho klienta dokud neodpojí.
pub async fn handle_client(scene: Arc<Mutex<WindowScene>>, stream: UnixStream) {
    let mut conn = AirProtoConn::from_stream(stream);
    while let Ok(msg) = conn.recv::<ClientMessage>().await {
        let mut s = scene.lock().unwrap();
        match msg {
            ClientMessage::WindowCreate { id, size } => s.add_window(id, size),
            ClientMessage::WindowDestroy { id } => s.remove_window(id),
            ClientMessage::WindowResize { id, size } => s.resize_window(id, size),
            ClientMessage::BufferCommit { id, data } => s.commit_buffer(id, data),
        }
    }
}

/// Spustí IPC server — přijímá spojení a pro každé spustí handle_client task.
pub async fn run_server(scene: Arc<Mutex<WindowScene>>, socket_path: &Path) -> std::io::Result<()> {
    let listener = UnixListener::bind(socket_path)?;
    loop {
        let (stream, _) = listener.accept().await?;
        let scene = scene.clone();
        tokio::spawn(handle_client(scene, stream));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use airproto::types::{Size, WindowId};
    use airproto::AirProtoConn;
    use tempfile::TempDir;

    #[tokio::test]
    async fn handle_client_window_create() {
        let scene = Arc::new(Mutex::new(WindowScene::new()));
        let dir = TempDir::new().unwrap();
        let socket_path = dir.path().join("test.sock");

        let listener = tokio::net::UnixListener::bind(&socket_path).unwrap();

        let server_scene = scene.clone();
        let server = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            handle_client(server_scene, stream).await;
        });

        let mut conn = AirProtoConn::connect(&socket_path).await.unwrap();
        conn.send(&ClientMessage::WindowCreate {
            id: WindowId(1),
            size: Size {
                width: 800,
                height: 600,
            },
        })
        .await
        .unwrap();
        drop(conn); // zavře spojení → handle_client loop skončí

        server.await.unwrap();

        let s = scene.lock().unwrap();
        assert!(s.get(WindowId(1)).is_some());
        assert_eq!(s.get(WindowId(1)).unwrap().size.width, 800);
    }

    #[tokio::test]
    async fn handle_client_window_destroy() {
        let scene = Arc::new(Mutex::new(WindowScene::new()));
        {
            scene.lock().unwrap().add_window(
                WindowId(1),
                Size {
                    width: 100,
                    height: 100,
                },
            );
        }
        let dir = TempDir::new().unwrap();
        let socket_path = dir.path().join("test2.sock");

        let listener = tokio::net::UnixListener::bind(&socket_path).unwrap();

        let server_scene = scene.clone();
        let server = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            handle_client(server_scene, stream).await;
        });

        let mut conn = AirProtoConn::connect(&socket_path).await.unwrap();
        conn.send(&ClientMessage::WindowDestroy { id: WindowId(1) })
            .await
            .unwrap();
        drop(conn);

        server.await.unwrap();

        assert!(scene.lock().unwrap().get(WindowId(1)).is_none());
    }

    #[tokio::test]
    async fn handle_client_multiple_messages() {
        let scene = Arc::new(Mutex::new(WindowScene::new()));
        let dir = TempDir::new().unwrap();
        let socket_path = dir.path().join("test3.sock");

        let listener = tokio::net::UnixListener::bind(&socket_path).unwrap();

        let server_scene = scene.clone();
        let server = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            handle_client(server_scene, stream).await;
        });

        let mut conn = AirProtoConn::connect(&socket_path).await.unwrap();
        conn.send(&ClientMessage::WindowCreate {
            id: WindowId(10),
            size: Size {
                width: 640,
                height: 480,
            },
        })
        .await
        .unwrap();
        conn.send(&ClientMessage::WindowResize {
            id: WindowId(10),
            size: Size {
                width: 1280,
                height: 720,
            },
        })
        .await
        .unwrap();
        drop(conn);

        server.await.unwrap();

        let s = scene.lock().unwrap();
        let w = s.get(WindowId(10)).unwrap();
        assert_eq!(w.size.width, 1280);
        assert_eq!(w.size.height, 720);
    }

    #[tokio::test]
    async fn run_server_accepts_multiple_clients() {
        let scene = Arc::new(Mutex::new(WindowScene::new()));
        let dir = TempDir::new().unwrap();
        let socket_path = dir.path().join("test4.sock");

        let server_scene = scene.clone();
        let server_path = socket_path.clone();
        let _server = tokio::spawn(async move {
            run_server(server_scene, &server_path).await.ok();
        });

        // Krátká pauza pro start serveru
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;

        // Klient 1
        let mut conn1 = AirProtoConn::connect(&socket_path).await.unwrap();
        conn1
            .send(&ClientMessage::WindowCreate {
                id: WindowId(1),
                size: Size {
                    width: 100,
                    height: 100,
                },
            })
            .await
            .unwrap();
        drop(conn1);

        // Klient 2
        let mut conn2 = AirProtoConn::connect(&socket_path).await.unwrap();
        conn2
            .send(&ClientMessage::WindowCreate {
                id: WindowId(2),
                size: Size {
                    width: 200,
                    height: 200,
                },
            })
            .await
            .unwrap();
        drop(conn2);

        tokio::time::sleep(std::time::Duration::from_millis(20)).await;

        let s = scene.lock().unwrap();
        assert!(s.get(WindowId(1)).is_some());
        assert!(s.get(WindowId(2)).is_some());
    }
}
