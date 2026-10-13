use crate::depth::DepthMap;
use agi_core::{CameraPose, PinholeCamera, Point3D};
use nalgebra::Vector3;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DensePoint {
    pub position: Point3D,
    pub normal: Vector3<f64>,
    pub color_rgb: [u8; 3],
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DensePointCloud {
    pub points: Vec<DensePoint>,
}

#[derive(Debug, Clone)]
pub struct FusionOptions {
    pub min_consistent_views: usize,
    pub depth_tolerance_m: f64,
    pub min_confidence: f64,
}

pub struct DepthMapFusion {
    pub options: FusionOptions,
}

impl DepthMapFusion {
    pub fn new(options: FusionOptions) -> Self {
        Self { options }
    }

    /// Fuses multiple depth maps into a consistent dense 3D point cloud.
    pub fn fuse(
        &self,
        views: &[(&PinholeCamera, &CameraPose, &DepthMap)],
    ) -> Option<DensePointCloud> {
        if views.is_empty() {
            return None;
        }

        let mut fused_points = Vec::new();

        for (v_idx, &(cam, pose, depth_map)) in views.iter().enumerate() {
            let w = depth_map.width;
            let h = depth_map.height;

            for y in (5..(h - 5)).step_by(2) {
                for x in (5..(w - 5)).step_by(2) {
                    let d = depth_map.get_depth(x, y);
                    if d <= 0.05 {
                        continue;
                    }

                    // Unproject pixel (x, y) with depth d (optical axis distance) to 3D world space
                    let x_norm = (x as f64 - cam.cx) / cam.fx;
                    let y_norm = (y as f64 - cam.cy) / cam.fy;
                    let pt_cam = Point3D::new(x_norm * d, y_norm * d, d);
                    let pt_world = pose.camera_to_world(&pt_cam);

                    // Consistency check across other views
                    let mut consistent_count = 1;

                    for (other_idx, &(_other_cam, other_pose, other_depth)) in
                        views.iter().enumerate()
                    {
                        if other_idx == v_idx {
                            continue;
                        }

                        if let Ok(other_px) = other_pose.project_world_point(&pt_world) {
                            let ox = other_px.x.round() as usize;
                            let oy = other_px.y.round() as usize;

                            if ox < other_depth.width && oy < other_depth.height {
                                let od = other_depth.get_depth(ox, oy);
                                let other_pt_cam = other_pose.world_to_camera(&pt_world);
                                let expected_depth = other_pt_cam.z;

                                if (od - expected_depth).abs() <= self.options.depth_tolerance_m {
                                    consistent_count += 1;
                                }
                            }
                        }
                    }

                    if consistent_count >= self.options.min_consistent_views {
                        fused_points.push(DensePoint {
                            position: pt_world,
                            normal: Vector3::new(0.0, 0.0, 1.0),
                            color_rgb: [160, 130, 95], // archaeological soil pigment
                            confidence: 0.95,
                        });
                    }
                }
            }
        }

        Some(DensePointCloud {
            points: fused_points,
        })
    }
}
