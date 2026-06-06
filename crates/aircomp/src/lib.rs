pub mod ipc_server;
pub mod rect_pipeline;
pub mod renderer;
pub mod scene;
pub mod shell_chrome;
pub mod surface;
pub mod window;

use std::sync::{Arc, Mutex};

pub use scene::WindowScene;
pub use window::WindowState;

pub fn run() {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    let scene = Arc::new(Mutex::new(WindowScene::new()));
    let socket_path = std::path::PathBuf::from("/tmp/aircomp.sock");
    let _ = std::fs::remove_file(&socket_path);

    let ipc_scene = scene.clone();
    rt.spawn(async move {
        ipc_server::run_server(ipc_scene, &socket_path)
            .await
            .expect("IPC server failed");
    });

    surface::run_event_loop(scene);
}

pub async fn run_headless(scene: Arc<Mutex<WindowScene>>) {
    renderer::run_headless(scene).await;
}
