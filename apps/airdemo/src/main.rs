//! airdemo — demo IPC klient pro AirComp.
//!
//! Připojí se k /tmp/aircomp.sock, vytvoří okno 640×480,
//! odešle statický checkerboard pixel buffer, pak čeká na WindowClose.

use airproto::types::{Size, WindowId};
use airproto::{AirProtoConn, ClientMessage, ServerMessage};
use std::path::Path;

/// Vygeneruje 640×480 RGBA checkerboard s 8px čtverci.
/// Černé čtverce: [0, 0, 0, 255], bílé: [255, 255, 255, 255].
fn generate_checkerboard(width: u32, height: u32, square_size: u32) -> Vec<u8> {
    let mut pixels = Vec::with_capacity((width * height * 4) as usize);
    for y in 0..height {
        for x in 0..width {
            let tile_x = x / square_size;
            let tile_y = y / square_size;
            let white = (tile_x + tile_y) % 2 == 0;
            let val = if white { 255u8 } else { 0u8 };
            pixels.extend_from_slice(&[val, val, val, 255]);
        }
    }
    pixels
}

#[tokio::main]
async fn main() {
    let socket_path = Path::new("/tmp/aircomp.sock");

    println!("airdemo: připojuji se k {}...", socket_path.display());
    let mut conn = AirProtoConn::connect(socket_path)
        .await
        .expect("Nepodařilo se připojit k aircomp. Je aircomp spuštěn?");
    println!("airdemo: připojeno.");

    let window_id = WindowId(1);
    let width = 640u32;
    let height = 480u32;

    // Vytvoříme okno
    conn.send(&ClientMessage::WindowCreate {
        id: window_id,
        size: Size { width, height },
    })
    .await
    .expect("WindowCreate selhalo");
    println!("airdemo: WindowCreate odesláno ({}×{}).", width, height);

    // Vygenerujeme a odešleme checkerboard
    let pixels = generate_checkerboard(width, height, 8);
    conn.send(&ClientMessage::BufferCommit {
        id: window_id,
        data: pixels,
    })
    .await
    .expect("BufferCommit selhalo");
    println!("airdemo: BufferCommit odesláno ({} bytů).", width * height * 4);

    // Čekáme na WindowClose od compositu
    println!("airdemo: čekám na WindowClose (zavři aircomp okno)...");
    loop {
        match conn.recv::<ServerMessage>().await {
            Ok(ServerMessage::WindowClose { window_id: id }) if id == window_id => {
                println!("airdemo: WindowClose přijato, konec.");
                break;
            }
            Ok(_) => {} // jiná událost, ignoruj
            Err(_) => {
                println!("airdemo: spojení ukončeno.");
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checkerboard_correct_size() {
        let pixels = generate_checkerboard(640, 480, 8);
        assert_eq!(pixels.len(), 640 * 480 * 4);
    }

    #[test]
    fn checkerboard_first_pixel_is_white() {
        let pixels = generate_checkerboard(8, 8, 8);
        // tile (0,0) — white
        assert_eq!(&pixels[0..4], &[255, 255, 255, 255]);
    }

    #[test]
    fn checkerboard_second_tile_is_black() {
        let pixels = generate_checkerboard(16, 8, 8);
        // tile (1,0) — black, pixel at x=8, y=0 → index 8*4=32
        assert_eq!(&pixels[32..36], &[0, 0, 0, 255]);
    }

    #[test]
    fn checkerboard_alpha_always_255() {
        let pixels = generate_checkerboard(16, 16, 8);
        for i in 0..(16 * 16) {
            assert_eq!(pixels[i * 4 + 3], 255, "alpha != 255 at pixel {}", i);
        }
    }
}
