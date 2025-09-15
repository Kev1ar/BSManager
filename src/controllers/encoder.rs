use opencv::{core, imgcodecs, prelude::*};

pub fn encode_to_jpeg(frame: &core::Mat, quality: i32) -> opencv::Result<Vec<u8>> {
    let mut buf = core::Vector::new();
    let params = core::Vector::from(vec![imgcodecs::IMWRITE_JPEG_QUALITY, quality]);
    imgcodecs::imencode(".jpg", frame, &mut buf, &params)?;
    Ok(buf.to_vec())
}
