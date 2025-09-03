use std::fs::{self, File};
use std::io::Write;
use chrono::Local;
use v4l::buffer::Type;
use v4l::io::traits::CaptureStream;
use v4l::video::Capture;
use v4l::FourCC;
use v4l::io::mmap::Stream;

pub fn capture_and_save(device_index: usize) -> Result<String, Box<dyn std::error::Error>> {
    println!("Orange Pi IA Controller Started...");
    let folder_path = "src/images";
    fs::create_dir_all(folder_path)?;

    // Open camera device
    let dev = v4l::Device::new(device_index)?;
    println!("Orange Pi IA Controller Started...");
    // Set format to MJPEG 640x480
    let mut fmt = dev.format()?;
    fmt.width = 640;
    fmt.height = 480;
    println!("Orange Pi IA Controller Started...");
    fmt.fourcc = FourCC::new(b"MJPG");
    dev.set_format(&fmt)?;
    println!("Orange Pi IA Controller Started...");
    // Create a memory-mapped capture stream
    let mut stream = Stream::with_buffers(&dev, v4l::buffer::Type::VideoCapture, 4)?;
    println!("fail..");
    let (data, _meta) = stream.next()?; // `data` now contains JPEG-compressed bytes
    println!("Orange Pi IA Controller Started...");
    // Timestamped filename
    let timestamp = Local::now().format("%Y%m%d_%H%M%S");
    let filename = format!("capture_{}.jpg", timestamp);
    let file_path = format!("{}/{}", folder_path, filename);
    println!("Orange Pi IA Controller Started...");
    // Save frame directly (already JPEG)
    let mut file = File::create(&file_path)?;
    file.write_all(&data)?;
    println!("Orange Pi IA Controller Started...");
    Ok(file_path)
}
