use agi_core::{CameraPose, PinholeCamera, Point2D, Point3D};
use nalgebra::DMatrix;

/// Triangulates a 3D point from two camera views using Linear DLT (Direct Linear Transform).
pub fn triangulate_dlt(
    cam1: &PinholeCamera,
    pose1: &CameraPose,
    pt1: &Point2D,
    cam2: &PinholeCamera,
    pose2: &CameraPose,
    pt2: &Point2D,
) -> Option<Point3D> {
    // Unproject distorted pixel coordinates to normalized camera ray coordinates (z=1)
    let ray1 = cam1.unproject(pt1);
    let ray2 = cam2.unproject(pt2);

    let nx1 = ray1.direction.x / ray1.direction.z;
    let ny1 = ray1.direction.y / ray1.direction.z;

    let nx2 = ray2.direction.x / ray2.direction.z;
    let ny2 = ray2.direction.y / ray2.direction.z;

    // Projection matrices for normalized coordinates: P = [R^T | -R^T * C]
    let r1_t = pose1.rotation.to_rotation_matrix().transpose();
    let t1 = -(r1_t * pose1.position.coords);

    let r2_t = pose2.rotation.to_rotation_matrix().transpose();
    let t2 = -(r2_t * pose2.position.coords);

    // Construct 4x4 DLT linear system A * X = 0
    let mut a = DMatrix::<f64>::zeros(4, 4);

    // Row 0: x1 * P1_row2 - P1_row0
    // Row 1: y1 * P1_row2 - P1_row1
    for col in 0..3 {
        a[(0, col)] = nx1 * r1_t[(2, col)] - r1_t[(0, col)];
        a[(1, col)] = ny1 * r1_t[(2, col)] - r1_t[(1, col)];
    }
    a[(0, 3)] = nx1 * t1[2] - t1[0];
    a[(1, 3)] = ny1 * t1[2] - t1[1];

    // Row 2: x2 * P2_row2 - P2_row0
    // Row 3: y2 * P2_row2 - P2_row1
    for col in 0..3 {
        a[(2, col)] = nx2 * r2_t[(2, col)] - r2_t[(0, col)];
        a[(3, col)] = ny2 * r2_t[(2, col)] - r2_t[(1, col)];
    }
    a[(2, 3)] = nx2 * t2[2] - t2[0];
    a[(3, 3)] = ny2 * t2[2] - t2[1];

    // Solve using SVD of A (nullspace vector)
    let svd = a.svd(true, true);
    let v_t = svd.v_t?;
    let x_hom = v_t.row(3);

    let w = x_hom[3];
    if w.abs() < 1e-12 {
        return None;
    }

    Some(Point3D::new(x_hom[0] / w, x_hom[1] / w, x_hom[2] / w))
}
