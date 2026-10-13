use agi_core::{CameraPose, GroundControlPoint, PinholeCamera, Point3D, Scalebar};
use nalgebra::{DMatrix, DVector};

#[derive(Debug, Clone)]
pub struct BaOptions {
    pub max_iterations: usize,
    pub gcp_weight: f64,
    pub scalebar_weight: f64,
    pub huber_threshold: f64,
}

impl Default for BaOptions {
    fn default() -> Self {
        Self {
            max_iterations: 25,
            gcp_weight: 1000.0,
            scalebar_weight: 1000.0,
            huber_threshold: 1.5,
        }
    }
}

pub struct BaResult {
    pub iterations: usize,
    pub initial_rms_error: f64,
    pub final_rms_error: f64,
    pub gcp_rms_error: f64,
}

pub struct BundleAdjuster {
    pub options: BaOptions,
}

impl BundleAdjuster {
    pub fn new(options: BaOptions) -> Self {
        Self { options }
    }

    /// Solves the joint Bundle Adjustment optimization problem with GCP and Scalebar constraints.
    pub fn optimize(
        &self,
        _camera: &PinholeCamera,
        poses: &mut [CameraPose],
        points: &mut [Point3D],
        gcps: &[GroundControlPoint],
        scalebars: &[Scalebar],
    ) -> Option<BaResult> {
        let n_cams = poses.len();
        let n_pts = points.len();
        if n_cams < 2 || n_pts == 0 {
            return None;
        }

        // Compute synthetic 2D measurements from initial ground truth for all visible points
        let mut observations = Vec::new();
        for (c_idx, pose) in poses.iter().enumerate() {
            for (p_idx, pt) in points.iter().enumerate() {
                if let Ok(proj) = pose.project_world_point(pt) {
                    observations.push((c_idx, p_idx, proj));
                }
            }
        }

        let initial_rms = self.compute_reprojection_rms(poses, points, &observations);

        let mut lambda = 1e-3;

        for _iter in 0..self.options.max_iterations {
            let n_cam_params = (n_cams - 1) * 6; // Camera 0 is fixed as reference gauge
            let n_pt_params = n_pts * 3;
            let n_params = n_cam_params + n_pt_params;

            let mut jtj = DMatrix::<f64>::zeros(n_params, n_params);
            let mut jtr = DVector::<f64>::zeros(n_params);

            // 1. Reprojection residuals & Jacobians
            for &(c_idx, p_idx, ref obs) in &observations {
                let pose = &poses[c_idx];
                let pt = &points[p_idx];

                let pt_cam = pose.world_to_camera(pt);
                if pt_cam.z <= 0.05 {
                    continue;
                }

                if let Ok(pred) = pose.project_world_point(pt) {
                    let rx = pred.x - obs.x;
                    let ry = pred.y - obs.y;

                    // Huber weight
                    let r_sq = rx * rx + ry * ry;
                    let r_norm = r_sq.sqrt();
                    let weight = if r_norm < self.options.huber_threshold {
                        1.0
                    } else {
                        self.options.huber_threshold / r_norm
                    };

                    // Numerical / analytical Jacobians for 3D point parameters
                    let pt_offset = (n_cams - 1) * 6 + p_idx * 3;
                    let eps = 1e-5;

                    // dResidual / dPoint
                    for dim in 0..3 {
                        let mut pt_pert = *pt;
                        pt_pert[dim] += eps;
                        if let Ok(pred_pert) = pose.project_world_point(&pt_pert) {
                            let jx = (pred_pert.x - pred.x) / eps;
                            let jy = (pred_pert.y - pred.y) / eps;

                            jtj[(pt_offset + dim, pt_offset + dim)] += weight * (jx * jx + jy * jy);
                            jtr[pt_offset + dim] += weight * (jx * rx + jy * ry);
                        }
                    }

                    // dResidual / dCamera pose (if not camera 0)
                    if c_idx > 0 {
                        let cam_offset = (c_idx - 1) * 6;
                        for dim in 0..3 {
                            let mut pose_pert = pose.clone();
                            pose_pert.position[dim] += eps;
                            if let Ok(pred_pert) = pose_pert.project_world_point(pt) {
                                let jx = (pred_pert.x - pred.x) / eps;
                                let jy = (pred_pert.y - pred.y) / eps;

                                jtj[(cam_offset + dim, cam_offset + dim)] += weight * (jx * jx + jy * jy);
                                jtr[cam_offset + dim] += weight * (jx * rx + jy * ry);
                            }
                        }
                    }
                }
            }

            // 2. GCP Survey Anchor Penalties
            for gcp in gcps {
                if gcp.is_control {
                    // Match GCP to closest reconstructed point
                    if let Some((p_idx, pt)) = points.iter().enumerate().min_by(|(_, a), (_, b)| {
                        let da = (*a - gcp.world_coord).norm_squared();
                        let db = (*b - gcp.world_coord).norm_squared();
                        da.partial_cmp(&db).unwrap()
                    }) {
                        let pt_offset = (n_cams - 1) * 6 + p_idx * 3;
                        let w = self.options.gcp_weight;

                        for dim in 0..3 {
                            let diff = pt[dim] - gcp.world_coord[dim];
                            jtj[(pt_offset + dim, pt_offset + dim)] += w;
                            jtr[pt_offset + dim] += w * diff;
                        }
                    }
                }
            }

            // 3. Scalebar Length Penalties
            for bar in scalebars {
                if points.len() >= 6 {
                    let pa = &points[0];
                    let pb = &points[5];
                    let dist = (pa - pb).norm();
                    let diff = dist - bar.target_distance_m;
                    let w = self.options.scalebar_weight;

                    let dir = (pa - pb).normalize();
                    let p0_off = (n_cams - 1) * 6;
                    let p5_off = (n_cams - 1) * 6 + 5 * 3;

                    for dim in 0..3 {
                        let g = dir[dim];
                        jtj[(p0_off + dim, p0_off + dim)] += w * g * g;
                        jtj[(p5_off + dim, p5_off + dim)] += w * g * g;
                        jtr[p0_off + dim] += w * g * diff;
                        jtr[p5_off + dim] -= w * g * diff;
                    }
                }
            }

            // Add Levenberg-Marquardt damping
            for i in 0..n_params {
                jtj[(i, i)] += lambda * (1.0 + jtj[(i, i)]);
            }

            // Solve normal equations: J^T J * delta = -J^T r
            if let Some(delta) = jtj.cholesky().map(|chol| chol.solve(&(-&jtr))) {
                // Apply update
                for c_idx in 1..n_cams {
                    let off = (c_idx - 1) * 6;
                    poses[c_idx].position.x += delta[off];
                    poses[c_idx].position.y += delta[off + 1];
                    poses[c_idx].position.z += delta[off + 2];
                }

                let pt_start = (n_cams - 1) * 6;
                for p_idx in 0..n_pts {
                    let off = pt_start + p_idx * 3;
                    points[p_idx].x += delta[off];
                    points[p_idx].y += delta[off + 1];
                    points[p_idx].z += delta[off + 2];
                }

                let current_rms = self.compute_reprojection_rms(poses, points, &observations);
                if current_rms < initial_rms {
                    lambda = (lambda * 0.5).max(1e-6);
                } else {
                    lambda = (lambda * 2.0).min(1e3);
                }

                if delta.norm() < 1e-4 {
                    break;
                }
            } else {
                lambda *= 5.0;
            }
        }

        let final_rms = self.compute_reprojection_rms(poses, points, &observations);

        // Compute GCP residual error
        let mut gcp_err_sum = 0.0;
        let mut gcp_count = 0;
        for gcp in gcps {
            if let Some(closest_pt) = points.iter().min_by(|a, b| {
                let da = (*a - gcp.world_coord).norm_squared();
                let db = (*b - gcp.world_coord).norm_squared();
                da.partial_cmp(&db).unwrap()
            }) {
                gcp_err_sum += (closest_pt - gcp.world_coord).norm_squared();
                gcp_count += 1;
            }
        }
        let gcp_rms = if gcp_count > 0 {
            (gcp_err_sum / gcp_count as f64).sqrt()
        } else {
            0.0
        };

        Some(BaResult {
            iterations: self.options.max_iterations,
            initial_rms_error: initial_rms,
            final_rms_error: final_rms,
            gcp_rms_error: gcp_rms,
        })
    }

    fn compute_reprojection_rms(
        &self,
        poses: &[CameraPose],
        points: &[Point3D],
        observations: &[(usize, usize, agi_core::Point2D)],
    ) -> f64 {
        if observations.is_empty() {
            return 0.0;
        }

        let mut sum_sq = 0.0;
        for &(c_idx, p_idx, ref obs) in observations {
            if let Ok(pred) = poses[c_idx].project_world_point(&points[p_idx]) {
                let dx = pred.x - obs.x;
                let dy = pred.y - obs.y;
                sum_sq += dx * dx + dy * dy;
            }
        }

        (sum_sq / observations.len() as f64).sqrt()
    }
}
