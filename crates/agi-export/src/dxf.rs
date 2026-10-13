use agi_core::{GroundControlPoint, Point2D, Point3D};
use agi_ortho::ContourLine;

/// AutoCAD DXF Writer conforming to AutoCAD R12 / R2000 ASCII format.
#[derive(Default)]
pub struct DxfWriter {
    entities: Vec<String>,
}

impl DxfWriter {
    pub fn new() -> Self {
        Self {
            entities: Vec::new(),
        }
    }

    /// Adds a 3D polyline to specified layer.
    pub fn add_polyline_3d(&mut self, layer: &str, points: &[Point3D]) {
        if points.is_empty() {
            return;
        }

        let mut s = String::new();
        s.push_str(&format!("  0\nPOLYLINE\n  8\n{}\n 66\n1\n 70\n8\n", layer));
        for pt in points {
            s.push_str(&format!(
                "  0\nVERTEX\n  8\n{}\n 10\n{:.4}\n 20\n{:.4}\n 30\n{:.4}\n 70\n32\n",
                layer, pt.x, pt.y, pt.z
            ));
        }
        s.push_str("  0\nSEQEND\n");
        self.entities.push(s);
    }

    /// Adds a 2D line segment to specified layer.
    pub fn add_line(&mut self, layer: &str, p1: Point2D, p2: Point2D) {
        let s = format!(
            "  0\nLINE\n  8\n{}\n 10\n{:.4}\n 20\n{:.4}\n 30\n0.0\n 11\n{:.4}\n 21\n{:.4}\n 31\n0.0\n",
            layer, p1.x, p1.y, p2.x, p2.y
        );
        self.entities.push(s);
    }

    /// Adds an archaeological contour line (10cm or 50cm index contour).
    pub fn add_contour(&mut self, contour: &ContourLine) {
        let layer = if contour.is_index {
            "CONTOURS_50CM"
        } else {
            "CONTOURS_10CM"
        };

        let pts3d: Vec<Point3D> = contour
            .points
            .iter()
            .map(|p| Point3D::new(p.x, p.y, contour.elevation_m))
            .collect();
        self.add_polyline_3d(layer, &pts3d);
    }

    /// Adds a Ground Control Point (GCP) marker with crosshair and text label.
    pub fn add_gcp_marker(&mut self, gcp: &GroundControlPoint) {
        let x = gcp.world_coord.x;
        let y = gcp.world_coord.y;
        let z = gcp.world_coord.z;

        // Crosshair point
        let point_ent = format!(
            "  0\nPOINT\n  8\nGCPS\n 10\n{:.4}\n 20\n{:.4}\n 30\n{:.4}\n",
            x, y, z
        );
        self.entities.push(point_ent);

        // Text label
        let text_ent = format!(
            "  0\nTEXT\n  8\nGCPS\n 10\n{:.4}\n 20\n{:.4}\n 30\n{:.4}\n 40\n0.20\n  1\n{}\n",
            x + 0.1,
            y + 0.1,
            z,
            gcp.label
        );
        self.entities.push(text_ent);
    }

    /// Adds the formal Archaeological Investigation Title Block.
    pub fn add_title_block(
        &mut self,
        site_name: &str,
        trench_id: &str,
        crs_name: &str,
        surveyor: &str,
    ) {
        let title_content = format!(
            "SITE: {} | TRENCH: {} | CRS: {} | SURVEYOR: {}",
            site_name, trench_id, crs_name, surveyor
        );

        let ent = format!(
            "  0\nTEXT\n  8\nTITLE_BLOCK\n 10\n0.0\n 20\n0.0\n 30\n0.0\n 40\n0.50\n  1\n{}\n",
            title_content
        );
        self.entities.push(ent);
    }

    /// Serializes entire DXF document into AutoCAD ASCII standard.
    pub fn write_to_string(&self) -> String {
        let mut out = String::new();

        // 1. HEADER SECTION
        out.push_str("  0\nSECTION\n  2\nHEADER\n  9\n$ACADVER\n  1\nAC1009\n  0\nENDSEC\n");

        // 2. TABLES SECTION (Layer Definitions)
        out.push_str("  0\nSECTION\n  2\nTABLES\n  0\nTABLE\n  2\nLAYER\n 70\n6\n");

        let layers = [
            ("TRENCH_OUTLINE", 1),  // Red
            ("STRATIGRAPHY_AA", 6), // Magenta
            ("CONTOURS_10CM", 2),   // Yellow
            ("CONTOURS_50CM", 30),  // Brown
            ("GCPS", 4),            // Cyan
            ("TITLE_BLOCK", 7),     // White
        ];

        for (lname, color) in layers {
            out.push_str(&format!(
                "  0\nLAYER\n  2\n{}\n 70\n0\n 62\n{}\n  6\nCONTINUOUS\n",
                lname, color
            ));
        }

        out.push_str("  0\nENDTAB\n  0\nENDSEC\n");

        // 3. BLOCKS SECTION
        out.push_str("  0\nSECTION\n  2\nBLOCKS\n  0\nENDSEC\n");

        // 4. ENTITIES SECTION
        out.push_str("  0\nSECTION\n  2\nENTITIES\n");
        for ent in &self.entities {
            out.push_str(ent);
        }
        out.push_str("  0\nENDSEC\n");

        // 5. EOF
        out.push_str("  0\nEOF\n");

        out
    }
}
