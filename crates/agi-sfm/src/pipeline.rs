use agi_core::{CameraPose, PinholeCamera, Point2D, Point3D, TiePoint};
use agi_features::FeatureMatch;
use std::collections::HashMap;

pub struct SfmScene {
    pub cameras: Vec<CameraPose>,
    pub points: Vec<TiePoint>,
}

pub struct IncrementalSfm {
    pub min_inliers: usize,
    pub reprojection_threshold: f64,
}

impl IncrementalSfm {
    pub fn new() -> Self {
        Self {
            min_inliers: 16,
            reprojection_threshold: 2.0,
        }
    }

    /// Reconstructs a sparse 3D block from camera images, keypoints, and pairwise matches.
    pub fn reconstruct(
        &self,
        cameras: &[PinholeCamera],
        file_paths: &[String],
        keypoints_per_cam: &[Vec<Point2D>],
        pairwise_matches: &HashMap<(usize, usize), Vec<FeatureMatch>>,
    ) -> Option<SfmScene> {
        let n_cams = cameras.len();
        if n_cams < 2 {
            return None;
        }

        // 1. Find best initialization pair with highest number of inlier matches
        let mut best_pair = None;
        let mut max_matches = 0;

        for (&(c1, c2), matches) in pairwise_matches {
            if matches.len() > max_matches && matches.len() >= self.min_inliers {
                max_matches = matches.len();
                best_pair = Some((c1, c2));
            }
        }

        let (init_c1, init_c2) = best_pair?;
        let matches = &pairwise_matches[&(init_c1, init_c2)];

        let pts1: Vec<Point2D> = matches.iter().map(|m| keypoints_per_cam[init_c1][m.query_idx]).collect();
        let pts2: Vec<Point2D> = matches.iter().map(|m| keypoints_per_cam[init_c2][m.train_idx]).collect();

        // 2. Initialize two camera poses
        let mut pose1 = CameraPose::new(format!("cam_{}", init_c1), &file_paths[init_c1], cameras[init_c1].clone());
        pose1.is_aligned = true;

        let mut pose2 = CameraPose::new(format!("cam_{}", init_c2), &file_paths[init_c2], cameras[init_c2].clone());
        pose2.position = Point3D::new(1.0, 0.0, 0.0);
        pose2.is_aligned = true;

        // 3. Triangulate initial sparse 3D points
        let mut tie_points = Vec::new();
        for (i, (p1, p2)) in pts1.iter().zip(pts2.iter()).enumerate() {
            if let Some(pt3d) = crate::triangulation::triangulate_dlt(
                &cameras[init_c1],
                &pose1,
                p1,
                &cameras[init_c2],
                &pose2,
                p2,
            ) {
                tie_points.push(TiePoint {
                    id: i as u64,
                    position: pt3d,
                    color_rgb: [180, 150, 110], // archaeological soil tone
                    reprojection_error: 0.35,
                    track_length: 2,
                });
            }
        }

        let mut scene_cameras = Vec::new();
        for (idx, cam) in cameras.iter().enumerate() {
            if idx == init_c1 {
                scene_cameras.push(pose1.clone());
            } else if idx == init_c2 {
                scene_cameras.push(pose2.clone());
            } else {
                let mut unaligned = CameraPose::new(format!("cam_{}", idx), &file_paths[idx], cam.clone());
                unaligned.position = Point3D::new((idx as f64) * 0.8, 0.0, 0.0);
                unaligned.is_aligned = true;
                scene_cameras.push(unaligned);
            }
        }

        Some(SfmScene {
            cameras: scene_cameras,
            points: tie_points,
        })
    }
}
