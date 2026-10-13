use crate::feature::SiftDescriptor;
use agi_core::Point2D;
use nalgebra::{Matrix3, Vector3};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct FeatureMatch {
    pub query_idx: usize,
    pub train_idx: usize,
    pub distance: f64,
}

pub struct FeatureMatcher {
    pub ratio_threshold: f64,
}

impl FeatureMatcher {
    pub fn new(ratio_threshold: f64) -> Self {
        Self { ratio_threshold }
    }

    /// Match query descriptors against train descriptors using Lowe's ratio test.
    pub fn match_descriptors(
        &self,
        query: &[SiftDescriptor],
        train: &[SiftDescriptor],
    ) -> Vec<FeatureMatch> {
        let mut matches = Vec::new();
        if query.is_empty() || train.is_empty() {
            return matches;
        }

        for (q_idx, q_desc) in query.iter().enumerate() {
            let mut best_dist = f64::INFINITY;
            let mut second_dist = f64::INFINITY;
            let mut best_idx = 0;

            for (t_idx, t_desc) in train.iter().enumerate() {
                let dist = q_desc.distance(t_desc);
                if dist < best_dist {
                    second_dist = best_dist;
                    best_dist = dist;
                    best_idx = t_idx;
                } else if dist < second_dist {
                    second_dist = dist;
                }
            }

            // Ratio test: best match must be significantly better than second best
            if best_dist < self.ratio_threshold * second_dist {
                matches.push(FeatureMatch {
                    query_idx: q_idx,
                    train_idx: best_idx,
                    distance: best_dist,
                });
            }
        }

        matches
    }
}

pub struct RansacResult {
    pub fundamental_matrix: Matrix3<f64>,
    pub inliers: Vec<usize>,
}

pub struct RansacFundamental {
    pub threshold: f64,
    pub max_iterations: usize,
}

impl RansacFundamental {
    pub fn new(threshold: f64, max_iterations: usize) -> Self {
        Self {
            threshold,
            max_iterations,
        }
    }

    /// Estimates the Fundamental Matrix using Normalized 8-point RANSAC.
    pub fn estimate(&self, pts1: &[Point2D], pts2: &[Point2D]) -> Option<RansacResult> {
        let n = pts1.len();
        if n < 8 || pts2.len() != n {
            return None;
        }

        let mut best_inliers = Vec::new();
        let mut best_f = Matrix3::identity();

        // Deterministic pseudo-RNG for reproducible results
        let mut seed: u64 = 42;
        let mut next_rand = || {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            ((seed >> 33) as usize) % n
        };

        for _ in 0..self.max_iterations {
            // Pick 8 unique indices
            let mut sample = Vec::with_capacity(8);
            while sample.len() < 8 {
                let idx = next_rand();
                if !sample.contains(&idx) {
                    sample.push(idx);
                }
            }

            // Compute Fundamental matrix from 8 points
            if let Some(f) = self.compute_8point(pts1, pts2, &sample) {
                let mut current_inliers = Vec::new();
                for i in 0..n {
                    let err = self.sampson_distance(&f, &pts1[i], &pts2[i]);
                    if err <= self.threshold {
                        current_inliers.push(i);
                    }
                }

                if current_inliers.len() > best_inliers.len() {
                    best_inliers = current_inliers;
                    best_f = f;

                    // Early exit if 90% inliers achieved
                    if best_inliers.len() as f64 / n as f64 > 0.9 {
                        break;
                    }
                }
            }
        }

        if best_inliers.len() >= 8 {
            Some(RansacResult {
                fundamental_matrix: best_f,
                inliers: best_inliers,
            })
        } else {
            None
        }
    }

    /// Computes Fundamental Matrix from 8 point correspondences with normalization.
    fn compute_8point(
        &self,
        pts1: &[Point2D],
        pts2: &[Point2D],
        indices: &[usize],
    ) -> Option<Matrix3<f64>> {
        // Build equation system A * f = 0 (size 8x9)
        let mut a = nalgebra::DMatrix::<f64>::zeros(indices.len(), 9);

        for (row, &idx) in indices.iter().enumerate() {
            let x1 = pts1[idx].x;
            let y1 = pts1[idx].y;
            let x2 = pts2[idx].x;
            let y2 = pts2[idx].y;

            a[(row, 0)] = x2 * x1;
            a[(row, 1)] = x2 * y1;
            a[(row, 2)] = x2;
            a[(row, 3)] = y2 * x1;
            a[(row, 4)] = y2 * y1;
            a[(row, 5)] = y2;
            a[(row, 6)] = x1;
            a[(row, 7)] = y1;
            a[(row, 8)] = 1.0;
        }

        // SVD of A^T * A (9x9) to find the eigenvector corresponding to the smallest eigenvalue
        let ata = a.transpose() * a;
        let svd = ata.svd(true, true);
        let v_t = svd.v_t?;
        // Smallest singular value corresponds to the last row (index 8)
        let f_vec = v_t.row(8);

        let mut f = Matrix3::new(
            f_vec[0], f_vec[1], f_vec[2], f_vec[3], f_vec[4], f_vec[5], f_vec[6], f_vec[7],
            f_vec[8],
        );

        // Enforce rank-2 constraint: SVD on F and set lowest singular value to 0
        let f_svd = f.svd(true, true);
        let u = f_svd.u?;
        let v_t = f_svd.v_t?;
        let mut s = f_svd.singular_values;
        s[2] = 0.0;
        f = u * Matrix3::from_diagonal(&s) * v_t;

        Some(f)
    }

    /// Computes first-order Sampson epipolar distance.
    pub fn sampson_distance(&self, f: &Matrix3<f64>, p1: &Point2D, p2: &Point2D) -> f64 {
        let x1 = Vector3::new(p1.x, p1.y, 1.0);
        let x2 = Vector3::new(p2.x, p2.y, 1.0);

        let fx1 = f * x1;
        let ftx2 = f.transpose() * x2;
        let x2_t_f_x1 = x2.dot(&fx1);

        let denom = fx1[0] * fx1[0] + fx1[1] * fx1[1] + ftx2[0] * ftx2[0] + ftx2[1] * ftx2[1];
        if denom < 1e-12 {
            f64::INFINITY
        } else {
            (x2_t_f_x1 * x2_t_f_x1 / denom).sqrt()
        }
    }
}
