use image::{DynamicImage, GenericImageView, ImageBuffer, Rgba, RgbaImage};
use image::imageops::FilterType;
use imageproc::drawing::draw_hollow_rect_mut;
use imageproc::rect::Rect;
use ndarray::{Array, Array4, Axis};
use onnxruntime::{environment::Environment, session::Session, session::SessionBuilder, tensor::OrtOwnedTensor};
use once_cell::sync::OnceCell;
use std::path::Path;
use std::sync::Arc;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum YoloError {
    #[error("ONNX error: {0}")]
    Onnx(#[from] onnxruntime::OrtError),
    #[error("Image error: {0}")]
    Img(#[from] image::ImageError),
    #[error("Invalid model output shape: {0:?}")]
    BadOutput(Vec<i64>),
    #[error("Other: {0}")]
    Other(String),
}

#[derive(Clone, Debug)]
pub struct Detection {
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
    pub score: f32,
    pub class_id: i32,
}

pub struct YoloDetector {
    session: Session,
    input_width: u32,
    input_height: u32,
    conf_threshold: f32,
    iou_threshold: f32,
}

static ORT_ENV: OnceCell<Arc<Environment>> = OnceCell::new();

impl YoloDetector {
    pub fn new<P: AsRef<Path>>(model_path: P) -> Result<Self, YoloError> {
        let env = ORT_ENV.get_or_try_init(|| {
            Environment::builder()
                .with_name("bsdmanager-yolo")
                .with_log_level(onnxruntime::LoggingLevel::Warning)
                .build()
                .map(Arc::new)
        })?.clone();

        let session = SessionBuilder::new(&env)?
            .with_optimization_level(onnxruntime::GraphOptimizationLevel::All)?
            .with_model_from_file(model_path)?;

        Ok(Self {
            session,
            input_width: 640,
            input_height: 640,
            conf_threshold: 0.25,
            iou_threshold: 0.45,
        })
    }

    pub fn with_thresholds(mut self, conf: f32, iou: f32) -> Self {
        self.conf_threshold = conf;
        self.iou_threshold = iou;
        self
    }

    pub fn infer_on_image_path<P: AsRef<Path>>(&self, image_path: P) -> Result<(Vec<Detection>, RgbaImage), YoloError> {
        let img = image::open(image_path)?;
        self.infer(img)
    }

    pub fn infer(&self, img: DynamicImage) -> Result<(Vec<Detection>, RgbaImage), YoloError> {
        // Preprocess
        let (tensor, scale, pad_x, pad_y, resized_rgba) = self.preprocess(&img)?;

        // Run
        let input_name = self.session.inputs[0].name.clone();
        let output = self.session.run(vec![(&input_name, tensor.view())])?;

        // Extract output tensor (assume single output)
        let output_any = output.into_iter().next().ok_or_else(|| YoloError::Other("No outputs from model".into()))?;
        // We try to map to f32 tensor
        let ort_tensor: OrtOwnedTensor<f32, _> = output_any.try_extract()?;
        let shape = ort_tensor.view().shape().iter().map(|d| *d as i64).collect::<Vec<_>>();

        // Postprocess
        let preds = self.postprocess(ort_tensor, &shape, scale, pad_x, pad_y, self.input_width as f32, self.input_height as f32)?;

        // Draw
        let annotated = Self::draw_detections(&resized_rgba, &preds);
        Ok((preds, annotated))
    }

    fn preprocess(&self, img: &DynamicImage) -> Result<(Array4<f32>, f32, f32, f32, RgbaImage), YoloError> {
        // Letterbox could be added; to keep it beginner-friendly we do a simple resize to 640x640
        let resized = img.resize_exact(self.input_width, self.input_height, FilterType::Lanczos3);
        let rgb = resized.to_rgb8();
        let mut chw = Array::zeros((1, 3, self.input_height as usize, self.input_width as usize));
        for (y, row) in rgb.rows().enumerate() {
            for (x, p) in row.enumerate() {
                chw[[0, 0, y, x]] = (p[0] as f32) / 255.0;
                chw[[0, 1, y, x]] = (p[1] as f32) / 255.0;
                chw[[0, 2, y, x]] = (p[2] as f32) / 255.0;
            }
        }
        // Since we resized directly, scale is simply the ratio to original
        let scale_x = self.input_width as f32 / img.width() as f32;
        let scale_y = self.input_height as f32 / img.height() as f32;
        // Use the min; for direct resize, we will later invert with original dims
        let scale = scale_x.min(scale_y);
        let pad_x = 0.0;
        let pad_y = 0.0;
        let rgba: RgbaImage = ImageBuffer::from_fn(self.input_width, self.input_height, |x, y| {
            let p = rgb.get_pixel(x, y);
            Rgba([p[0], p[1], p[2], 255])
        });
        Ok((chw, scale, pad_x, pad_y, rgba))
    }

    fn postprocess(
        &self,
        ort: OrtOwnedTensor<f32, ndarray::Dim<ndarray::IxDynImpl>>,
        shape: &Vec<i64>,
        _scale: f32,
        _pad_x: f32,
        _pad_y: f32,
        orig_w: f32,
        orig_h: f32,
    ) -> Result<Vec<Detection>, YoloError> {
        // Expect either [1, N, 85] or [1, 85, N]
        let view = ort.view();
        let detections = if shape.len() == 3 && shape[0] == 1 {
            if shape[2] >= 6 && shape[1] > 6 {
                // [1, 85, N] -> transpose to [N, 85]
                let n = shape[2] as usize;
                let c = shape[1] as usize;
                let mut dets = Vec::new();
                for i in 0..n {
                    let x = view[[0, 0, i]];
                    let y = view[[0, 1, i]];
                    let w = view[[0, 2, i]];
                    let h = view[[0, 3, i]];
                    let obj = view[[0, 4, i]];
                    // Single-class: use obj as score
                    let score = obj;
                    if score < self.conf_threshold { continue; }
                    let (x1, y1, x2, y2) = Self::xywh_to_xyxy(x, y, w, h, orig_w, orig_h);
                    dets.push(Detection { x1, y1, x2, y2, score, class_id: 0 });
                }
                dets
            } else if shape[1] >= 6 && shape[2] > 6 {
                // [1, N, 85]
                let n = shape[1] as usize;
                let mut dets = Vec::new();
                for i in 0..n {
                    let x = view[[0, i, 0]];
                    let y = view[[0, i, 1]];
                    let w = view[[0, i, 2]];
                    let h = view[[0, i, 3]];
                    let obj = view[[0, i, 4]];
                    let score = obj;
                    if score < self.conf_threshold { continue; }
                    let (x1, y1, x2, y2) = Self::xywh_to_xyxy(x, y, w, h, orig_w, orig_h);
                    dets.push(Detection { x1, y1, x2, y2, score, class_id: 0 });
                }
                dets
            } else {
                return Err(YoloError::BadOutput(shape.clone()));
            }
        } else {
            return Err(YoloError::BadOutput(shape.clone()));
        };

        Ok(self.nms(detections, self.iou_threshold))
    }

    fn xywh_to_xyxy(x: f32, y: f32, w: f32, h: f32, orig_w: f32, orig_h: f32) -> (f32, f32, f32, f32) {
        // Assuming predictions are in pixels relative to resized image; we clamp to original dims
        let x1 = (x - w / 2.0).max(0.0).min(orig_w - 1.0);
        let y1 = (y - h / 2.0).max(0.0).min(orig_h - 1.0);
        let x2 = (x + w / 2.0).max(0.0).min(orig_w - 1.0);
        let y2 = (y + h / 2.0).max(0.0).min(orig_h - 1.0);
        (x1, y1, x2, y2)
    }

    fn iou(a: &Detection, b: &Detection) -> f32 {
        let x1 = a.x1.max(b.x1);
        let y1 = a.y1.max(b.y1);
        let x2 = a.x2.min(b.x2);
        let y2 = a.y2.min(b.y2);
        let inter = ((x2 - x1).max(0.0)) * ((y2 - y1).max(0.0));
        let area_a = (a.x2 - a.x1).max(0.0) * (a.y2 - a.y1).max(0.0);
        let area_b = (b.x2 - b.x1).max(0.0) * (b.y2 - b.y1).max(0.0);
        inter / (area_a + area_b - inter + 1e-6)
    }

    fn nms(&self, mut dets: Vec<Detection>, iou_thresh: f32) -> Vec<Detection> {
        dets.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        let mut keep: Vec<Detection> = Vec::new();
        while let Some(det) = dets.pop() {
            let mut suppressed = false;
            for kept in &keep {
                if Self::iou(&det, kept) > iou_thresh { suppressed = true; break; }
            }
            if !suppressed { keep.push(det); }
        }
        keep
    }

    pub fn draw_detections(img: &RgbaImage, dets: &Vec<Detection>) -> RgbaImage {
        let mut out = img.clone();
        let color = Rgba([255u8, 0u8, 0u8, 255u8]); // red boxes
        for d in dets {
            let x = d.x1.max(0.0) as i32;
            let y = d.y1.max(0.0) as i32;
            let w = (d.x2 - d.x1).max(0.0) as u32;
            let h = (d.y2 - d.y1).max(0.0) as u32;
            if w > 0 && h > 0 {
                let rect = Rect::at(x, y).of_size(w, h);
                draw_hollow_rect_mut(&mut out, rect, color);
            }
        }
        out
    }

    pub fn save_image<P: AsRef<Path>>(img: &RgbaImage, path: P) -> Result<(), YoloError> {
        img.save(path)?;
        Ok(())
    }
}
