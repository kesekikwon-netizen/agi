pub mod dxf;
pub mod geotiff;
pub mod submission;

pub use dxf::DxfWriter;
pub use geotiff::{GeoTiffWriter, WorldFile};
pub use submission::{compute_sha256, SubmissionMetadata, SubmissionPacker};

#[cfg(test)]
mod tests {
    use super::*;
    use agi_core::{CoordinateSystem, GroundControlPoint, Point2D, Point3D};
    use agi_ortho::{ContourLine, Orthomosaic};

    #[test]
    fn test_autocad_dxf_archaeological_layers_export() {
        let mut dxf = DxfWriter::new();

        // 1. Add Archaeological Title Block
        dxf.add_title_block(
            "Gwangryeong-ri Archaeological Site",
            "Trench A-01",
            "EPSG:5179 (Korea 2000 Unified CS)",
            "Cultural Heritage Investigator",
        );

        // 2. Add Trench Outline (TRENCH_OUTLINE layer)
        let trench_boundary = vec![
            Point3D::new(100.0, 200.0, 35.0),
            Point3D::new(110.0, 200.0, 35.0),
            Point3D::new(110.0, 210.0, 34.8),
            Point3D::new(100.0, 210.0, 34.8),
            Point3D::new(100.0, 200.0, 35.0),
        ];
        dxf.add_polyline_3d("TRENCH_OUTLINE", &trench_boundary);

        // 3. Add 10cm Contours (CONTOURS_10CM and CONTOURS_50CM)
        let c1 = ContourLine {
            elevation_m: 34.90,
            points: vec![Point2D::new(100.0, 205.0), Point2D::new(110.0, 205.1)],
            is_index: false,
        };
        let c2 = ContourLine {
            elevation_m: 35.00,
            points: vec![Point2D::new(100.0, 202.0), Point2D::new(110.0, 202.1)],
            is_index: true,
        };
        dxf.add_contour(&c1);
        dxf.add_contour(&c2);

        // 4. Add Ground Control Point (GCPS layer)
        let gcp = GroundControlPoint {
            label: "GCP-01".into(),
            world_coord: Point3D::new(105.0, 205.0, 34.95),
            crs: CoordinateSystem::KoreanUtmK5179,
            is_control: true,
            accuracy_m: 0.003,
            image_measurements: vec![],
        };
        dxf.add_gcp_marker(&gcp);

        let dxf_string = dxf.write_to_string();

        assert!(
            dxf_string.contains("SECTION\n  2\nHEADER"),
            "Must contain HEADER section"
        );
        assert!(
            dxf_string.contains("SECTION\n  2\nTABLES"),
            "Must contain TABLES section"
        );
        assert!(
            dxf_string.contains("TRENCH_OUTLINE"),
            "Must define TRENCH_OUTLINE layer"
        );
        assert!(
            dxf_string.contains("CONTOURS_10CM"),
            "Must define CONTOURS_10CM layer"
        );
        assert!(
            dxf_string.contains("CONTOURS_50CM"),
            "Must define CONTOURS_50CM layer"
        );
        assert!(dxf_string.contains("GCPS"), "Must define GCPS layer");
        assert!(dxf_string.contains("GCP-01"), "Must write GCP marker label");
        assert!(
            dxf_string.contains("Gwangryeong-ri Archaeological Site"),
            "Must write site title block"
        );
        assert!(dxf_string.contains("EOF"), "Must end with DXF EOF token");
    }

    #[test]
    fn test_world_file_and_geotiff_metadata_generation() {
        let ortho = Orthomosaic {
            width: 1000,
            height: 800,
            gsd_m: 0.005, // 5mm / px
            min_x: 194800.0,
            min_y: 452100.0,
            max_x: 194805.0,
            max_y: 452104.0,
            rgb_data: vec![[150, 120, 90]; 1000 * 800],
            dem_data: vec![34.5; 1000 * 800],
        };

        let world_file = WorldFile::from_ortho(&ortho);
        let tfw_content = world_file.to_string();
        let lines: Vec<&str> = tfw_content.lines().collect();

        assert_eq!(lines.len(), 6, "ESRI World file must have exactly 6 lines");
        let x_scale: f64 = lines[0].parse().expect("Valid line 1 float");
        let rot1: f64 = lines[1].parse().expect("Valid line 2 float");
        let rot2: f64 = lines[2].parse().expect("Valid line 3 float");
        let y_scale: f64 = lines[3].parse().expect("Valid line 4 float");
        let upper_left_x: f64 = lines[4].parse().expect("Valid line 5 float");
        let upper_left_y: f64 = lines[5].parse().expect("Valid line 6 float");

        assert!(
            (x_scale - 0.005).abs() < 1e-6,
            "X scale must be positive GSD"
        );
        assert_eq!(rot1, 0.0, "Rotation term 1 must be 0.0");
        assert_eq!(rot2, 0.0, "Rotation term 2 must be 0.0");
        assert!(
            (y_scale + 0.005).abs() < 1e-6,
            "Y scale must be negative GSD"
        );
        assert!(
            (upper_left_x - (194800.0 + 0.0025)).abs() < 1e-4,
            "Center of upper left pixel X"
        );
        assert!(
            (upper_left_y - (452104.0 - 0.0025)).abs() < 1e-4,
            "Center of upper left pixel Y"
        );

        // GeoTIFF binary structure
        let tiff_bytes = GeoTiffWriter::write_ortho_tiff(&ortho, CoordinateSystem::KoreanUtmK5179)
            .expect("GeoTIFF byte stream creation must succeed");

        assert!(tiff_bytes.len() > 100, "GeoTIFF must contain valid bytes");
        // Verify little-endian TIFF signature (0x49 0x49 0x2A 0x00) -> "II*\0"
        assert_eq!(
            &tiff_bytes[0..4],
            b"II\x2A\x00",
            "TIFF magic header must match"
        );
    }

    #[test]
    fn test_national_heritage_submission_package_generation_with_sha256() {
        let meta = SubmissionMetadata {
            site_name: "Gwangryeong-ri Pit 1".into(),
            trench_id: "PIT-01".into(),
            investigation_agency: "Cultural Heritage Excavation Center".into(),
            surveyor: "Lead Archaeologist".into(),
            crs_code: "EPSG:5179".into(),
            gsd_m: 0.001, // 1mm GSD
            rms_reprojection_px: 0.28,
            gcp_rms_error_m: 0.002, // 2mm residual error
            photo_count: 48,
            point_count: 8_420_000,
        };

        let dummy_dxf = b"0\nSECTION\n2\nENTITIES\n0\nEOF\n";
        let dummy_tfw = b"0.001\n0.0\n0.0\n-0.001\n100.0\n200.0\n";
        let dummy_tif = b"II\x2A\x00\x08\x00\x00\x00";

        let files: Vec<(&str, &[u8])> = vec![
            ("04_CAD_DRAWINGS/Trench_01.dxf", dummy_dxf),
            ("05_ORTHOMOSAIC/Trench_01.tfw", dummy_tfw),
            ("05_ORTHOMOSAIC/Trench_01.tif", dummy_tif),
        ];

        let packer = SubmissionPacker::new();
        let manifest = packer.build_manifest(&files);

        assert!(manifest.contains("04_CAD_DRAWINGS/Trench_01.dxf"));
        assert!(manifest.contains("05_ORTHOMOSAIC/Trench_01.tfw"));
        assert!(manifest.contains("05_ORTHOMOSAIC/Trench_01.tif"));

        // Verify SHA-256 format: 64 hex characters followed by two spaces and path
        let lines: Vec<&str> = manifest.lines().collect();
        assert_eq!(lines.len(), 3);
        for line in lines {
            let parts: Vec<&str> = line.split("  ").collect();
            assert_eq!(parts.len(), 2);
            assert_eq!(
                parts[0].len(),
                64,
                "SHA-256 hash must be 64 hexadecimal characters"
            );
            assert!(parts[0].chars().all(|c| c.is_ascii_hexdigit()));
        }

        let report = packer.generate_provenance_report(&meta);
        assert!(report.contains("Gwangryeong-ri Pit 1"));
        assert!(report.contains("EPSG:5179"));
        assert!(report.contains("0.28 px"));
        assert!(report.contains("8,420,000"));
    }
}
