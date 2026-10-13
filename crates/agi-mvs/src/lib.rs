pub mod depth;
pub mod fusion;
pub mod mesh;

pub use depth::{DepthMap, PatchMatchOptions, PatchMatchStereo};
pub use fusion::{DensePoint, DensePointCloud, DepthMapFusion, FusionOptions};
pub use mesh::{Mesh3D, SurfaceReconstructor, Triangle};

#[cfg(test)]
mod tests {
    use super::*;
    use agi_core::{CameraPose, PinholeCamera, Point3D};
    use nalgebra::Vector3;

    #[test]
    fn test_zncc_patch_matching_precision() {
        let p1 = vec![10.0, 20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0, 90.0];
        // Identical patch
        let p2 = p1.clone();
        // Inverted / dissimilar patch
        let p3 = vec![90.0, 80.0, 70.0, 60.0, 50.0, 40.0, 30.0, 20.0, 10.0];

        let zncc_identical = depth::compute_zncc(&p1, &p2);
        let zncc_dissimilar = depth::compute_zncc(&p1, &p3);

        assert!(
            (zncc_identical - 1.0).abs() < 1e-6,
            "ZNCC for identical patch must be 1.0"
        );
        assert!(
            zncc_dissimilar < 0.0,
            "ZNCC for inverted patch must be negative"
        );
    }

    #[test]
    fn test_depth_map_fusion_into_dense_cloud() {
        let cam1 = PinholeCamera::new(100, 100, 100.0, 100.0, 50.0, 50.0);
        let cam2 = PinholeCamera::new(100, 100, 100.0, 100.0, 50.0, 50.0);

        let pose1 = CameraPose::new("cam1", "img1.jpg", cam1.clone());
        let mut pose2 = CameraPose::new("cam2", "img2.jpg", cam2.clone());
        pose2.position = Point3D::new(0.5, 0.0, 0.0);

        // Planar trench at depth Z = 3.0 meters
        let (w, h) = (100, 100);
        let depth_data1 = vec![3.0f64; w * h];
        let depth_data2 = vec![3.0f64; w * h];

        let depth_map1 = DepthMap::new(w, h, depth_data1, 0.95);
        let depth_map2 = DepthMap::new(w, h, depth_data2, 0.95);

        let fusion = DepthMapFusion::new(FusionOptions {
            min_consistent_views: 2,
            depth_tolerance_m: 0.005, // 5mm
            min_confidence: 0.8,
        });

        let cloud = fusion
            .fuse(&[(&cam1, &pose1, &depth_map1), (&cam2, &pose2, &depth_map2)])
            .expect("Fusion should succeed");

        assert!(
            cloud.points.len() > 1000,
            "Should reconstruct dense point cloud"
        );
        for pt in &cloud.points {
            assert!(
                (pt.position.z - 3.0).abs() < 0.005,
                "Depth error must be < 5mm"
            );
            assert!(pt.confidence >= 0.8);
        }
    }

    #[test]
    fn test_2d5_delaunay_mesh_reconstruction() {
        // Archaeological excavation trench grid: 5x5 grid with elevation variations
        let mut points = Vec::new();
        for y in 0..5 {
            for x in 0..5 {
                let px = (x as f64) * 0.5;
                let py = (y as f64) * 0.5;
                let pz = 1.0 + 0.1 * ((x + y) as f64);
                points.push(DensePoint {
                    position: Point3D::new(px, py, pz),
                    normal: Vector3::new(0.0, 0.0, 1.0),
                    color_rgb: [140, 110, 80],
                    confidence: 0.99,
                });
            }
        }

        let cloud = DensePointCloud { points };
        let reconstructor = SurfaceReconstructor::new();
        let mesh = reconstructor
            .reconstruct_2d5_delaunay(&cloud)
            .expect("Mesh reconstruction should succeed");

        assert!(
            mesh.triangles.len() >= 32,
            "5x5 grid should produce at least 32 triangles"
        );
        assert_eq!(mesh.vertices.len(), 25);

        // Verify positive surface area and upwards normals
        for tri in &mesh.triangles {
            let area = mesh.triangle_area(tri);
            assert!(area > 0.0, "Triangle area must be strictly positive");
            let normal = mesh.triangle_normal(tri);
            assert!(normal.z > 0.0, "Normal must point upwards (+Z)");
        }
    }
}
