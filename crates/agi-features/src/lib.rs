pub mod feature;
pub mod matching;
pub mod target;

pub use feature::{FeatureDetector, Keypoint, SiftDescriptor};
pub use matching::{FeatureMatch, FeatureMatcher, RansacFundamental};
pub use target::{CodedTarget, TargetDetector, TargetFormat};

#[cfg(test)]
mod tests {
    use super::*;
    use agi_core::Point2D;

    #[test]
    fn test_feature_matching_ratio_test() {
        // Query descriptors
        let d1 = SiftDescriptor::new(vec![1.0; 128]);
        let mut d2_vals = vec![0.0; 128];
        d2_vals[0..64].fill(1.0);
        let d2 = SiftDescriptor::new(d2_vals);

        // Train descriptors:
        // t1 is exact match for d1
        let t1 = SiftDescriptor::new(vec![1.0; 128]);

        // t2 and t3 are almost equally distant to d2 (distance ratio ~0.95 > 0.75) -> ratio test rejects d2
        let mut t2_vals = vec![0.0; 128];
        t2_vals[0..64].fill(0.9);
        t2_vals[64..128].fill(0.1);
        let t2 = SiftDescriptor::new(t2_vals);

        let mut t3_vals = vec![0.0; 128];
        t3_vals[0..64].fill(0.92);
        t3_vals[64..128].fill(0.08);
        let t3 = SiftDescriptor::new(t3_vals);

        let matcher = FeatureMatcher::new(0.75);
        let matches = matcher.match_descriptors(&[d1, d2], &[t1, t2, t3]);

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].query_idx, 0);
        assert_eq!(matches[0].train_idx, 0);
        assert!(matches[0].distance < 1e-4);
    }

    #[test]
    fn test_ransac_fundamental_matrix_synthetic() {
        // Synthetic corresponding points from 2 cameras
        let mut pts1 = Vec::new();
        let mut pts2 = Vec::new();

        // 30 true inlier correspondences with a horizontal baseline
        for i in 0..30 {
            let x = 100.0 + (i as f64) * 15.0;
            let y = 150.0 + ((i % 5) as f64) * 20.0;
            pts1.push(Point2D::new(x, y));
            // Disparity along x
            pts2.push(Point2D::new(x - 25.0, y + 2.0));
        }

        // Add 6 outliers
        pts1.push(Point2D::new(50.0, 50.0));
        pts2.push(Point2D::new(400.0, 300.0));
        pts1.push(Point2D::new(80.0, 20.0));
        pts2.push(Point2D::new(300.0, 80.0));

        let ransac = RansacFundamental::new(2.0, 1000);
        let result = ransac
            .estimate(&pts1, &pts2)
            .expect("RANSAC should find fundamental matrix");

        // Outliers should be rejected, inliers >= 28
        assert!(result.inliers.len() >= 28);
        assert!(!result.inliers.contains(&30)); // outlier index
        assert!(!result.inliers.contains(&31)); // outlier index
    }

    #[test]
    fn test_circular_coded_target_detection() {
        // Archaeological 14-bit target specification
        let detector = TargetDetector::new(TargetFormat::Circular14Bit);

        // Synthetic image patch with target centered at (100.4, 100.6)
        let (width, height) = (200, 200);
        let center_x = 100.4;
        let center_y = 100.6;
        let mut image_data = vec![240u8; width * height]; // light background

        // Render target concentric circles: black center bullseye radius 15
        for y in 0..height {
            for x in 0..width {
                let dx = x as f64 - center_x;
                let dy = y as f64 - center_y;
                let dist = (dx * dx + dy * dy).sqrt();
                if dist <= 16.0 {
                    image_data[y * width + x] = 20; // dark center
                } else if dist > 16.0 && dist <= 24.0 {
                    image_data[y * width + x] = 240; // white ring separator
                } else if dist > 24.0 && dist <= 38.0 {
                    // Coded bit segments
                    let angle = dy.atan2(dx);
                    let norm_angle = if angle < 0.0 {
                        angle + 2.0 * std::f64::consts::PI
                    } else {
                        angle
                    };
                    let bit_idx = (norm_angle / (2.0 * std::f64::consts::PI) * 14.0) as usize;
                    // Target code with alternating pattern
                    if (bit_idx % 2) == 0 {
                        image_data[y * width + x] = 20; // black code bit
                    } else {
                        image_data[y * width + x] = 240; // white code bit
                    }
                }
            }
        }

        let detected = detector.detect(&image_data, width, height);
        assert!(
            !detected.is_empty(),
            "Should detect the 14-bit coded target"
        );
        let target = &detected[0];
        println!(
            "Detected target center: ({}, {}), expected: ({}, {})",
            target.center.x, target.center.y, center_x, center_y
        );
        assert!(
            (target.center.x - center_x).abs() < 0.5,
            "Subpixel center X should match"
        );
        assert!(
            (target.center.y - center_y).abs() < 0.5,
            "Subpixel center Y should match"
        );
    }
}
