pub mod contour;
pub mod ortho;
pub mod section;

pub use contour::{ContourExtractor, ContourLine};
pub use ortho::{OrthoGenerator, Orthomosaic};
pub use section::{SectionPoint, SectionSlicer, StratigraphySection};

#[cfg(test)]
mod tests {
    use super::*;
    use agi_core::{Point2D, Point3D};
    use agi_mvs::{DensePoint, DensePointCloud};
    use nalgebra::Vector3;

    #[test]
    fn test_plan_orthomosaic_generation() {
        // Archaeological excavation trench: 2m x 2m area, elevation Z = 34.0m + 0.05 * x
        let mut points = Vec::new();
        let step = 0.05; // 5cm point spacing
        let mut x = 0.0;
        while x <= 2.001 {
            let mut y = 0.0;
            while y <= 2.001 {
                let z = 34.0 + 0.05 * x;
                points.push(DensePoint {
                    position: Point3D::new(x, y, z),
                    normal: Vector3::new(0.0, 0.0, 1.0),
                    color_rgb: [130, 95, 60], // Trench soil brown
                    confidence: 0.98,
                });
                y += step;
            }
            x += step;
        }

        let cloud = DensePointCloud { points };
        let gsd_m = 0.02; // 2cm / pixel
        let generator = OrthoGenerator::new(gsd_m);
        let ortho = generator
            .generate_from_pointcloud(&cloud)
            .expect("Orthomosaic generation must succeed");

        assert!(ortho.width >= 100, "2m at 2cm/px must be ~100px wide");
        assert!(ortho.height >= 100, "2m at 2cm/px must be ~100px high");
        assert_eq!(ortho.rgb_data.len(), ortho.width * ortho.height);
        assert_eq!(ortho.dem_data.len(), ortho.width * ortho.height);

        // Check DEM elevation at center (x=1.0, y=1.0) -> expected Z ~ 34.05m
        let center_z = ortho
            .get_elevation_at_world(1.0, 1.0)
            .expect("Elevation should exist");
        assert!(
            (center_z - 34.05).abs() < 0.02,
            "Elevation error must be < 2cm"
        );

        // Check color at center
        let color = ortho
            .get_color_at_world(1.0, 1.0)
            .expect("Color should exist");
        assert_eq!(color, [130, 95, 60]);
    }

    #[test]
    fn test_archaeological_stratigraphy_section_slicing() {
        // Vertical trench profile: 4m long along Y=2.0 from X=0.0 to X=4.0
        // Stratigraphy layers:
        // Layer I (topsoil): Z in [34.5, 35.0] RGB [100, 70, 40]
        // Layer II (burial layer): Z in [33.5, 34.5] RGB [60, 45, 30]
        // Layer III (bedrock): Z in [33.0, 33.5] RGB [160, 150, 130]
        let mut points = Vec::new();
        for xi in 0..80 {
            let px = (xi as f64) * 0.05; // 0.0 to 4.0m
            for zi in 0..40 {
                let pz = 33.0 + (zi as f64) * 0.05; // 33.0 to 35.0m
                                                    // Introduce slight noise in Y within trench wall buffer (+/- 3cm)
                let py = 2.0 + ((((xi + zi) % 7) as f64) - 3.0) * 0.01;

                let color = if pz >= 34.5 {
                    [100, 70, 40]
                } else if pz >= 33.5 {
                    [60, 45, 30]
                } else {
                    [160, 150, 130]
                };

                points.push(DensePoint {
                    position: Point3D::new(px, py, pz),
                    normal: Vector3::new(0.0, 1.0, 0.0),
                    color_rgb: color,
                    confidence: 0.95,
                });
            }
        }

        let cloud = DensePointCloud { points };
        let slicer = SectionSlicer::new();
        let p_start = Point2D::new(0.0, 2.0);
        let p_end = Point2D::new(4.0, 2.0);
        let buffer_m = 0.05; // +/- 5cm thickness
        let resolution_m = 0.01; // 1cm pixel resolution

        let section = slicer
            .slice_pointcloud(&cloud, p_start, p_end, buffer_m, resolution_m)
            .expect("Stratigraphy profile section slicing must succeed");

        assert!(
            (section.length_m - 4.0).abs() < 1e-3,
            "Section length must be 4.0m"
        );
        assert!(
            (section.min_elevation - 33.0).abs() < 0.05,
            "Min elevation must be ~33.0m"
        );
        assert!(
            (section.max_elevation - 35.0).abs() < 0.05,
            "Max elevation must be ~35.0m"
        );
        assert!(
            section.profile_points.len() > 1000,
            "Must capture section points"
        );

        // Verify elevation grid markers (33.0, 34.0, 35.0)
        assert!(section.elevation_grid_lines.contains(&33.0));
        assert!(section.elevation_grid_lines.contains(&34.0));
        assert!(section.elevation_grid_lines.contains(&35.0));

        // Verify raster image dimensions: 4m at 1cm/px = 400px wide, 2m at 1cm/px = 200px high
        assert_eq!(section.raster_width, 401);
        assert_eq!(section.raster_height, 201);
        assert_eq!(section.raster_image.len(), 401 * 201);
    }

    #[test]
    fn test_marching_squares_10cm_contour_extraction() {
        // DEM with elevation sloping from 34.40m to 35.20m over a 10m x 10m grid
        let width = 50;
        let height = 50;
        let gsd = 0.2; // 20cm/cell -> 10m x 10m
        let mut dem_data = vec![0.0f64; width * height];

        for y in 0..height {
            for x in 0..width {
                // Slope primarily in X direction
                dem_data[y * width + x] = 34.40 + 0.80 * (x as f64 / width as f64);
            }
        }

        let ortho = Orthomosaic {
            width,
            height,
            gsd_m: gsd,
            min_x: 100.0,
            min_y: 200.0,
            max_x: 110.0,
            max_y: 210.0,
            rgb_data: vec![[120, 100, 80]; width * height],
            dem_data,
        };

        let extractor = ContourExtractor::new();
        // 10cm contour interval (0.10m), index interval 0.50m (e.g. 35.0m)
        let contours = extractor.extract_contours(&ortho, 0.10, 0.50);

        assert!(
            !contours.is_empty(),
            "Contour extractor must generate contours"
        );

        // Verify that 35.0m is marked as an index contour (주곡선 / 계곡선)
        let index_contours: Vec<&ContourLine> = contours.iter().filter(|c| c.is_index).collect();
        assert!(!index_contours.is_empty(), "Index contours must exist");
        assert!(
            index_contours
                .iter()
                .any(|c| (c.elevation_m - 35.0).abs() < 1e-4),
            "35.0m must be an index contour"
        );

        // Verify contour coordinates lie within bounds [100, 110] x [200, 210]
        for contour in &contours {
            for pt in &contour.points {
                assert!(pt.x >= 99.9 && pt.x <= 110.1);
                assert!(pt.y >= 199.9 && pt.y <= 210.1);
            }
        }
    }
}
