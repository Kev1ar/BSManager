mod backend;
mod ai;
use backend::connection::Connection;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use tokio::time::{sleep, Duration};
use std::env;
use ai::yolo::YoloDetector;
use std::path::Path;
use std::fs;

#[tokio::main]
async fn main() {
    // Load env from src/.env if present
    let _ = dotenvy::from_path("src/.env");

    // let connection = Connection::new();
    // let connected = Arc::clone(&connection.connected);

    // // Wait until backend connects
    // Connection::wait_for_connection("0.0.0.0:5000", connected).await;

    // println!("connected...Run listener task now");

    // Optional: run YOLO demo if environment variables are set
    if let (Ok(model_path), Ok(image_path)) = (env::var("YOLO_MODEL"), env::var("YOLO_IMAGE")) {
        println!("Running YOLO demo...\n  Model: {}\n  Image: {}", model_path, image_path);
        match YoloDetector::new(&model_path) {
            Ok(detector) => {
                match detector.infer_on_image_path(&image_path) {
                    Ok((dets, annotated)) => {
                        let out_path = env::var("YOLO_OUT").unwrap_or_else(|_| "yolo_output.png".to_string());
                        if let Some(parent) = Path::new(&out_path).parent() { let _ = fs::create_dir_all(parent); }
                        if let Err(e) = YoloDetector::save_image(&annotated, &out_path) {
                            eprintln!("Failed to save annotated image: {}", e);
                        } else {
                            println!("YOLO detections: {}. Saved annotated image to {}", dets.len(), out_path);
                        }
                    }
                    Err(e) => eprintln!("YOLO inference failed: {}", e),
                }
            }
            Err(e) => eprintln!("Failed to initialize YOLO detector: {}", e),
        }
    } else {
        println!("YOLO demo skipped. Set YOLO_MODEL and YOLO_IMAGE env vars to enable.");
    }

    // Just keep program alive
    while connection.connected.load(Ordering::SeqCst) {
        sleep(Duration::from_secs(1)).await;
    }
}
