pub mod ipc_server;
pub mod renderer;
pub mod scene;
pub mod window;

use std::sync::{Arc, Mutex};

pub use scene::WindowScene;
pub use window::WindowState;

pub async fn run() {
    let scene = Arc::new(Mutex::new(WindowScene::new()));
    let socket_path = std::path::PathBuf::from("/tmp/aircomp.sock");

    let _ = std::fs::remove_file(&socket_path);

    let ipc_scene = scene.clone();
    let ipc_path = socket_path.clone();
    let ipc_task = tokio::spawn(async move {
        ipc_server::run_server(ipc_scene, &ipc_path)
            .await
            .expect("IPC server failed");
    });

    let render_scene = scene.clone();
    let render_task = tokio::spawn(async move {
        renderer::run_headless(render_scene).await;
    });

    tokio::select! {
        _ = ipc_task => {},
        _ = render_task => {},
    }
}
