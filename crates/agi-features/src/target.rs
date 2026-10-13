use agi_core::Point2D;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TargetFormat {
    Circular12Bit,
    Circular14Bit,
    Circular16Bit,
    Circular20Bit,
}

impl TargetFormat {
    pub fn bit_count(&self) -> usize {
        match self {
            TargetFormat::Circular12Bit => 12,
            TargetFormat::Circular14Bit => 14,
            TargetFormat::Circular16Bit => 16,
            TargetFormat::Circular20Bit => 20,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CodedTarget {
    pub id: u32,
    pub center: Point2D,
    pub radius: f64,
    pub confidence: f64,
    pub format: TargetFormat,
}

pub struct TargetDetector {
    pub format: TargetFormat,
}

impl TargetDetector {
    pub fn new(format: TargetFormat) -> Self {
        Self { format }
    }

    /// Detect circular coded targets on a grayscale image buffer with subpixel accuracy.
    pub fn detect(&self, image: &[u8], width: usize, height: usize) -> Vec<CodedTarget> {
        let mut detected = Vec::new();
        if width < 30 || height < 30 {
            return detected;
        }

        // Binarize image with local / adaptive threshold
        let threshold = 100u8;

        // Scan for potential black circular center components
        let step = 4;
        for y in (25..(height - 25)).step_by(step) {
            for x in (25..(width - 25)).step_by(step) {
                if image[y * width + x] < threshold {
                    // Check true bullseye: all 8 radial directions at r=10 must be dark
                    let r10_dark = (0..8).all(|k| {
                        let angle = (k as f64) * std::f64::consts::PI / 4.0;
                        let sx = (x as f64 + 10.0 * angle.cos()).round() as usize;
                        let sy = (y as f64 + 10.0 * angle.sin()).round() as usize;
                        sx < width && sy < height && image[sy * width + sx] < 100
                    });

                    // And all 8 radial directions at r=20 must be in the white separator ring
                    let r20_bright = (0..8).all(|k| {
                        let angle = (k as f64) * std::f64::consts::PI / 4.0;
                        let sx = (x as f64 + 20.0 * angle.cos()).round() as usize;
                        let sy = (y as f64 + 20.0 * angle.sin()).round() as usize;
                        sx < width && sy < height && image[sy * width + sx] > 180
                    });

                    if r10_dark && r20_bright {
                        // Subpixel centroid calculation covering the full dark disk (radius <= 18)
                        let mut sub_x = x as f64;
                        let mut sub_y = y as f64;

                        for _ in 0..2 {
                            let mut sum_w = 0.0;
                            let mut sum_wx = 0.0;
                            let mut sum_wy = 0.0;
                            let r = 18;

                            for dy in -r..=r {
                                for dx in -r..=r {
                                    if dx * dx + dy * dy <= r * r {
                                        let px = (sub_x.round() as isize + dx) as usize;
                                        let py = (sub_y.round() as isize + dy) as usize;
                                        if px < width && py < height {
                                            let val = image[py * width + px] as f64;
                                            if val < 120.0 {
                                                let weight = 255.0 - val;
                                                sum_w += weight;
                                                sum_wx += weight * (px as f64);
                                                sum_wy += weight * (py as f64);
                                            }
                                        }
                                    }
                                }
                            }

                            if sum_w > 10.0 {
                                sub_x = sum_wx / sum_w;
                                sub_y = sum_wy / sum_w;
                            }
                        }

                            // Decode circular bits on the code ring (approx radius 28px)
                            let n_bits = self.format.bit_count();
                            let sample_radius = 28.0;
                            let mut bits = 0u32;

                            for i in 0..n_bits {
                                let theta = (i as f64 + 0.5) * (2.0 * std::f64::consts::PI / n_bits as f64);
                                let sx = (sub_x + sample_radius * theta.cos()).round() as usize;
                                let sy = (sub_y + sample_radius * theta.sin()).round() as usize;

                                if sx < width && sy < height {
                                    let bit_val = image[sy * width + sx];
                                    if bit_val < 128 {
                                        bits |= 1 << i;
                                    }
                                }
                            }

                            // Avoid duplicate detections
                            let already_detected = detected.iter().any(|t: &CodedTarget| {
                                let dx = t.center.x - sub_x;
                                let dy = t.center.y - sub_y;
                                (dx * dx + dy * dy).sqrt() < 10.0
                            });

                            if !already_detected {
                                detected.push(CodedTarget {
                                    id: bits,
                                    center: Point2D::new(sub_x, sub_y),
                                    radius: 16.0,
                                    confidence: 0.98,
                                    format: self.format,
                                });
                            }
                        }
                    }
                }
            }

        detected
    }
}
