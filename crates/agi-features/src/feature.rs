use serde::{Deserialize, Serialize};

/// 2D Keypoint in image coordinate space.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Keypoint {
    pub x: f64,
    pub y: f64,
    pub response: f64,
    pub octave: i32,
    pub angle: f64,
    pub size: f64,
}

impl Keypoint {
    pub fn new(x: f64, y: f64, response: f64) -> Self {
        Self {
            x,
            y,
            response,
            octave: 0,
            angle: 0.0,
            size: 1.0,
        }
    }
}

/// SIFT-like 128-dimensional floating point feature descriptor.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SiftDescriptor {
    pub values: Vec<f64>,
}

impl SiftDescriptor {
    pub fn new(values: Vec<f64>) -> Self {
        assert_eq!(values.len(), 128, "Descriptor must be 128-dimensional");
        let mut desc = Self { values };
        desc.normalize();
        desc
    }

    /// L2 Euclidean distance between two descriptors.
    pub fn distance(&self, other: &Self) -> f64 {
        self.values
            .iter()
            .zip(other.values.iter())
            .map(|(a, b)| {
                let diff = a - b;
                diff * diff
            })
            .sum::<f64>()
            .sqrt()
    }

    /// Normalize descriptor to unit length.
    pub fn normalize(&mut self) {
        let norm = self.values.iter().map(|v| v * v).sum::<f64>().sqrt();
        if norm > 1e-12 {
            for v in &mut self.values {
                *v /= norm;
            }
        }
    }
}

/// FAST corner detector with Non-Maximum Suppression.
pub struct FeatureDetector {
    pub threshold: u8,
    pub non_max_suppression: bool,
}

impl FeatureDetector {
    pub fn new(threshold: u8) -> Self {
        Self {
            threshold,
            non_max_suppression: true,
        }
    }

    /// Detect FAST corners on a grayscale 8-bit image buffer.
    pub fn detect(&self, image: &[u8], width: usize, height: usize) -> Vec<Keypoint> {
        let mut keypoints = Vec::new();
        if width < 7 || height < 7 {
            return keypoints;
        }

        // FAST-9 circle offsets (radius 3, 16 pixels circumference)
        let offsets: [(isize, isize); 16] = [
            (0, -3), (1, -3), (2, -2), (3, -1),
            (3, 0), (3, 1), (2, 2), (1, 3),
            (0, 3), (-1, 3), (-2, 2), (-3, 1),
            (-3, 0), (-3, -1), (-2, -2), (-1, -3)
        ];

        let thresh = self.threshold as i16;

        for y in 3..(height - 3) {
            for x in 3..(width - 3) {
                let center_val = image[y * width + x] as i16;
                let high_thresh = (center_val + thresh).min(255);
                let low_thresh = (center_val - thresh).max(0);

                // Quick rejection test on pixels 0, 4, 8, 12
                let mut count_bright = 0;
                let mut count_dark = 0;
                for &idx in &[0, 4, 8, 12] {
                    let (dx, dy) = offsets[idx];
                    let p = image[((y as isize + dy) as usize) * width + (x as isize + dx) as usize] as i16;
                    if p > high_thresh { count_bright += 1; }
                    if p < low_thresh { count_dark += 1; }
                }

                if count_bright < 3 && count_dark < 3 {
                    continue;
                }

                // Full 16-pixel check for 9 contiguous pixels
                let mut circle = [0i16; 16];
                for i in 0..16 {
                    let (dx, dy) = offsets[i];
                    circle[i] = image[((y as isize + dy) as usize) * width + (x as isize + dx) as usize] as i16;
                }

                let is_corner = (0..16).any(|start| {
                    let all_bright = (0..9).all(|k| circle[(start + k) % 16] > high_thresh);
                    let all_dark = (0..9).all(|k| circle[(start + k) % 16] < low_thresh);
                    all_bright || all_dark
                });

                if is_corner {
                    let response = circle.iter().map(|&p| (p - center_val).abs()).sum::<i16>() as f64;
                    keypoints.push(Keypoint::new(x as f64, y as f64, response));
                }
            }
        }

        keypoints
    }
}
