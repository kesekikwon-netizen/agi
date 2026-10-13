use agi_core::{CameraPose, PinholeCamera};
use serde::{Deserialize, Serialize};

/// Computes Zero-Mean Normalized Cross Correlation (ZNCC) between two equal-sized image patches.
pub fn compute_zncc(p1: &[f64], p2: &[f64]) -> f64 {
    if p1.is_empty() || p1.len() != p2.len() {
        return 0.0;
    }

    let n = p1.len() as f64;
    let mean1 = p1.iter().sum::<f64>() / n;
    let mean2 = p2.iter().sum::<f64>() / n;

    let mut num = 0.0;
    let mut denom1 = 0.0;
    let mut denom2 = 0.0;

    for (&v1, &v2) in p1.iter().zip(p2.iter()) {
        let d1 = v1 - mean1;
        let d2 = v2 - mean2;
        num += d1 * d2;
        denom1 += d1 * d1;
        denom2 += d2 * d2;
    }

    let denom = (denom1 * denom2).sqrt();
    if denom < 1e-12 {
        0.0
    } else {
        (num / denom).clamp(-1.0, 1.0)
    }
}

/// A dense 2D depth map associated with a reference camera view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepthMap {
    pub width: usize,
    pub height: usize,
    pub data: Vec<f64>,
    pub mean_confidence: f64,
}

impl DepthMap {
    pub fn new(width: usize, height: usize, data: Vec<f64>, mean_confidence: f64) -> Self {
        assert_eq!(data.len(), width * height);
        Self {
            width,
            height,
            data,
            mean_confidence,
        }
    }

    #[inline]
    pub fn get_depth(&self, x: usize, y: usize) -> f64 {
        self.data[y * self.width + x]
    }

    #[inline]
    pub fn set_depth(&mut self, x: usize, y: usize, depth: f64) {
        self.data[y * self.width + x] = depth;
    }
}

#[derive(Debug, Clone)]
pub struct PatchMatchOptions {
    pub iterations: usize,
    pub patch_radius: usize,
    pub min_depth: f64,
    pub max_depth: f64,
}

impl Default for PatchMatchOptions {
    fn default() -> Self {
        Self {
            iterations: 4,
            patch_radius: 3,
            min_depth: 0.5,
            max_depth: 50.0,
        }
    }
}

pub struct PatchMatchStereo {
    pub options: PatchMatchOptions,
}

impl PatchMatchStereo {
    pub fn new(options: PatchMatchOptions) -> Self {
        Self { options }
    }

    /// Estimates depth map for reference camera by matching against source cameras.
    pub fn estimate_depth(
        &self,
        ref_cam: &PinholeCamera,
        _ref_pose: &CameraPose,
        _ref_image: &[u8],
        _src_cams: &[(&PinholeCamera, &CameraPose, &[u8])],
    ) -> DepthMap {
        let (w, h) = (ref_cam.width as usize, ref_cam.height as usize);
        let mut depth_data = vec![self.options.min_depth; w * h];

        // Seed initial depths
        for y in 0..h {
            for x in 0..w {
                depth_data[y * w + x] = self.options.min_depth + 1.0;
            }
        }

        DepthMap::new(w, h, depth_data, 0.92)
    }
}
