use agi_core::{CameraPose, PinholeCamera, Point2D, Point3D, RotationQuat};
use nalgebra::{Matrix3, UnitQuaternion, Vector3};

#[derive(Debug, Clone, PartialEq)]
pub struct EssentialMatrix {
    pub matrix: Matrix3<f64>,
}

impl EssentialMatrix {
    pub fn new(matrix: Matrix3<f64>) -> Self {
        Self { matrix }
    }

    /// Compute Essential Matrix directly from two known camera poses.
    pub fn from_poses(pose1: &CameraPose, pose2: &CameraPose) -> Self {
        let r1 = pose1.rotation.to_rotation_matrix();
        let r2 = pose2.rotation.to_rotation_matrix();

        // Relative rotation from cam1 to cam2: R_rel = R2^T * R1
        let r_rel = r2.transpose() * r1;

        // Relative translation in cam2 coordinates
        let t_world = pose2.position - pose1.position;
        let t_cam2 = r2.transpose() * t_world;

        // Skew-symmetric cross product matrix [t]_x
        let tx = Matrix3::new(
            0.0, -t_cam2.z, t_cam2.y,
            t_cam2.z, 0.0, -t_cam2.x,
            -t_cam2.y, t_cam2.x, 0.0,
        );

        let e = tx * r_rel;
        Self { matrix: e }
    }
}

/// Decomposes an Essential Matrix into relative rotation and translation using Cheirality test.
pub fn decompose_essential(
    essential: &EssentialMatrix,
    camera: &PinholeCamera,
    pts1: &[Point2D],
    pts2: &[Point2D],
) -> Option<(RotationQuat, Point3D)> {
    let svd = essential.matrix.svd(true, true);
    let u = svd.u?;
    let v_t = svd.v_t?;

    // Hartley-Zisserman W matrix
    let w = Matrix3::new(
        0.0, -1.0, 0.0,
        1.0, 0.0, 0.0,
        0.0, 0.0, 1.0,
    );

    let mut r1 = u * w * v_t;
    if r1.determinant() < 0.0 {
        r1 = -r1;
    }
    let mut r2 = u * w.transpose() * v_t;
    if r2.determinant() < 0.0 {
        r2 = -r2;
    }

    let u3 = u.column(2);
    let t_pos = Vector3::new(u3[0], u3[1], u3[2]);
    let t_neg = -t_pos;

    // 4 Hypotheses
    let hypotheses = [
        (r1, t_pos),
        (r1, t_neg),
        (r2, t_pos),
        (r2, t_neg),
    ];

    let pose1 = CameraPose::new("cam1", "img1.jpg", camera.clone());
    let mut best_count = 0;
    let mut best_hypothesis = None;

    for (rot_mat, t_vec) in hypotheses {
        let quat = UnitQuaternion::from_rotation_matrix(&nalgebra::Rotation3::from_matrix_unchecked(rot_mat));
        let mut pose2 = CameraPose::new("cam2", "img2.jpg", camera.clone());
        pose2.rotation = quat;
        pose2.position = Point3D::from(t_vec);

        let mut positive_depth_count = 0;
        for (p1, p2) in pts1.iter().zip(pts2.iter()) {
            if let Some(pt3d) = crate::triangulation::triangulate_dlt(camera, &pose1, p1, camera, &pose2, p2) {
                let pt_cam1 = pose1.world_to_camera(&pt3d);
                let pt_cam2 = pose2.world_to_camera(&pt3d);
                if pt_cam1.z > 0.0 && pt_cam2.z > 0.0 {
                    positive_depth_count += 1;
                }
            }
        }

        if positive_depth_count > best_count {
            best_count = positive_depth_count;
            best_hypothesis = Some((quat, Point3D::from(t_vec)));
        }
    }

    best_hypothesis
}
