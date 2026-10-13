pub mod ba;
pub mod epipolar;
pub mod pipeline;
pub mod triangulation;

pub use ba::{BaOptions, BaResult, BundleAdjuster};
pub use epipolar::{decompose_essential, EssentialMatrix};
pub use pipeline::{IncrementalSfm, SfmScene};
pub use triangulation::triangulate_dlt;

#[cfg(test)]
mod tests {
    use super::*;
    use agi_core::{
        CameraPose, CoordinateSystem, GroundControlPoint, PinholeCamera, Point3D, Scalebar,
    };
    use nalgebra::UnitQuaternion;

    #[test]
    fn test_dlt_triangulation_precision() {
        let cam1 = PinholeCamera::new(1000, 1000, 1000.0, 1000.0, 500.0, 500.0);
        let cam2 = PinholeCamera::new(1000, 1000, 1000.0, 1000.0, 500.0, 500.0);

        let pose1 = CameraPose::new("cam1", "img1.jpg", cam1.clone());
        let mut pose2 = CameraPose::new("cam2", "img2.jpg", cam2.clone());
        pose2.position = Point3D::new(1.0, 0.0, 0.0);

        let true_point = Point3D::new(0.5, 0.2, 5.0);

        let proj1 = pose1.project_world_point(&true_point).unwrap();
        let proj2 = pose2.project_world_point(&true_point).unwrap();

        let reconstructed = triangulate_dlt(&cam1, &pose1, &proj1, &cam2, &pose2, &proj2)
            .expect("Triangulation should succeed");

        assert!((reconstructed.x - true_point.x).abs() < 1e-4);
        assert!((reconstructed.y - true_point.y).abs() < 1e-4);
        assert!((reconstructed.z - true_point.z).abs() < 1e-4);
    }

    #[test]
    fn test_essential_matrix_decomposition_cheirality() {
        let cam = PinholeCamera::new(1000, 1000, 1000.0, 1000.0, 500.0, 500.0);

        let pose1 = CameraPose::new("cam1", "img1.jpg", cam.clone());
        let mut pose2 = CameraPose::new("cam2", "img2.jpg", cam.clone());
        let rot2 = UnitQuaternion::from_euler_angles(0.0, 0.05, 0.0);
        pose2.rotation = rot2;
        pose2.position = Point3D::new(1.0, 0.0, 0.0);

        // 10 3D points in front of both cameras
        let mut pts1 = Vec::new();
        let mut pts2 = Vec::new();
        for i in 0..10 {
            let p = Point3D::new(-1.0 + (i as f64) * 0.2, 0.5, 4.0 + (i as f64) * 0.1);
            pts1.push(pose1.project_world_point(&p).unwrap());
            pts2.push(pose2.project_world_point(&p).unwrap());
        }

        let essential = EssentialMatrix::from_poses(&pose1, &pose2);
        let (recovered_rot, recovered_t) = decompose_essential(&essential, &cam, &pts1, &pts2)
            .expect("Should decompose essential matrix with positive cheirality");

        let rot_cam2 = recovered_rot.inverse();
        let rot_error = rot_cam2.angle_to(&rot2);
        assert!(
            rot_error < 1e-4,
            "Recovered camera rotation must match rot2 within 1e-4 rad"
        );

        // Translation in world frame: R * t_cam
        let t_world = rot_cam2.transform_vector(&recovered_t.coords).normalize();
        assert!(
            (t_world.x - 1.0).abs() < 1e-4,
            "World baseline along X must be 1.0"
        );
        assert!(t_world.y.abs() < 1e-4);
        assert!(t_world.z.abs() < 1e-4);
    }

    #[test]
    fn test_bundle_adjustment_with_gcp_and_scalebar() {
        // Setup a 3-camera archaeological excavation block
        let cam = PinholeCamera::new(1200, 800, 1200.0, 1200.0, 600.0, 400.0);

        let mut poses = vec![
            CameraPose::new("cam0", "img0.jpg", cam.clone()),
            CameraPose::new("cam1", "img1.jpg", cam.clone()),
            CameraPose::new("cam2", "img2.jpg", cam.clone()),
        ];
        poses[1].position = Point3D::new(1.0, 0.0, 0.0);
        poses[2].position = Point3D::new(2.0, 0.0, 0.0);

        // 12 3D scene points (trench pit)
        let mut points_3d = Vec::new();
        for i in 0..12 {
            let x = 0.2 + (i as f64) * 0.15;
            let y = -0.5 + ((i % 3) as f64) * 0.3;
            let z = 3.5 + ((i % 4) as f64) * 0.1;
            points_3d.push(Point3D::new(x, y, z));
        }

        // 2 GCPs with ground truth survey coordinates
        let gcps = vec![
            GroundControlPoint {
                label: "GCP-01".into(),
                world_coord: points_3d[0],
                crs: CoordinateSystem::KoreanCentral5186,
                is_control: true,
                accuracy_m: 0.002,
                image_measurements: vec![],
            },
            GroundControlPoint {
                label: "GCP-02".into(),
                world_coord: points_3d[11],
                crs: CoordinateSystem::KoreanCentral5186,
                is_control: true,
                accuracy_m: 0.002,
                image_measurements: vec![],
            },
        ];

        // 1 Scalebar between point 0 and point 5 (length known)
        let bar_len = (points_3d[0] - points_3d[5]).norm();
        let scalebars = vec![Scalebar {
            label: "BAR-01".into(),
            marker1_label: "GCP-01".into(),
            marker2_label: "PT-05".into(),
            target_distance_m: bar_len,
            accuracy_m: 0.0005,
        }];

        let adjuster = BundleAdjuster::new(BaOptions {
            max_iterations: 30,
            gcp_weight: 1000.0,
            scalebar_weight: 1000.0,
            huber_threshold: 1.5,
        });

        // Add small perturbation to initial camera poses
        let mut perturbed_poses = poses.clone();
        perturbed_poses[1].position.x += 0.02;
        perturbed_poses[2].position.x -= 0.02;

        let result = adjuster
            .optimize(
                &cam,
                &mut perturbed_poses,
                &mut points_3d,
                &gcps,
                &scalebars,
            )
            .expect("Bundle Adjustment should converge");

        assert!(result.iterations > 0);
        assert!(
            result.final_rms_error < 0.5,
            "RMS reprojection error should be < 0.5 px"
        );
        assert!(
            result.gcp_rms_error < 0.005,
            "GCP residual error should be < 5 mm"
        );
    }
}
