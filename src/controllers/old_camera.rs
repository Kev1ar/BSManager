use opencv::{prelude::*, videoio, core, Result};

pub struct Camera {
    cap: videoio::VideoCapture,
}

impl Drop for Camera {
    fn drop(&mut self) {
        if let Err(e) = self.cap.release() {
            eprintln!("Failed to release camera: {:?}", e);
        } else {
            println!("Camera released");
        }
    }
}

impl Camera {
    pub fn new(index: i32, width: i32, height: i32, fps: i32) -> Result<Self> {
        let mut cap = videoio::VideoCapture::new(index, videoio::CAP_V4L2)?;
        cap.set(videoio::CAP_PROP_FRAME_WIDTH, width as f64)?;
        cap.set(videoio::CAP_PROP_FRAME_HEIGHT, height as f64)?;
        cap.set(videoio::CAP_PROP_FPS, fps as f64)?;

        if !cap.is_opened()? {
            return Err(opencv::Error::new(core::StsError, "Could not open camera"));
        }

        Ok(Camera { cap })
    }

    pub fn capture_frame(&mut self) -> Result<core::Mat> {
        let mut frame = core::Mat::default();
        self.cap.read(&mut frame)?;
        if frame.empty() {
            return Err(opencv::Error::new(core::StsError, "Empty frame"));
        }
        Ok(frame)
    }
}
