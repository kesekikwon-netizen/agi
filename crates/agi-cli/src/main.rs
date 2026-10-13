use agi_core::{engine_banner, Block, CameraPose, CoordinateSystem, PinholeCamera, Point3D};

fn main() {
    println!("{}", engine_banner());
    println!("Architecture: Pure Rust 100% Photogrammetry Core");

    // 1. Initialize an Archaeological Block (Metashape Chunk / ContextCapture Block)
    let crs = CoordinateSystem::KoreanCentral5186;
    let mut block = Block::new("block_01", "Gyeongju Trench 01", crs);
    println!(
        "\n[1] Block Initialized: {} ({})",
        block.name,
        crs.display_name()
    );

    // 2. Add an archeological camera
    let intrinsics = PinholeCamera::new(6000, 4000, 4500.0, 4500.0, 3000.0, 2000.0);
    let mut cam = CameraPose::new("DSC_0001", "samples/trench_01.jpg", intrinsics);
    cam.position = Point3D::new(0.0, 0.0, -5.0); // 5 meters above trench
    cam.is_aligned = true;
    block.cameras.push(cam);
    println!(
        "[2] Camera Added: {} [Aligned: {}]",
        block.cameras[0].id, block.cameras[0].is_aligned
    );

    // 3. Project an excavation feature point (0, 0, 0)
    let trench_feature = Point3D::new(0.0, 0.0, 0.0);
    if let Ok(pixel) = block.cameras[0].project_world_point(&trench_feature) {
        println!(
            "[3] Feature Projection: (0, 0, 0) -> Image Pixel: ({:.2}, {:.2})",
            pixel.x, pixel.y
        );
    }

    // 4. Geodetic Coordinate Conversion (EPSG:5186 -> EPSG:5179 submission)
    let gyeongju_lat = 35.838;
    let gyeongju_lon = 129.213;
    let crs_east = CoordinateSystem::KoreanEast5187;
    let crs_submit = CoordinateSystem::KoreanUtmK5179;

    let (e_work, n_work) = crs_east.forward(gyeongju_lat, gyeongju_lon).unwrap();
    let (e_submit, n_submit) = crs_east.transform_to(crs_submit, e_work, n_work).unwrap();

    println!("\n[4] Archaeological Geodetic Transformation:");
    println!(
        "  - Site: Gyeongju Historic Area ({:.4}° N, {:.4}° E)",
        gyeongju_lat, gyeongju_lon
    );
    println!(
        "  - Work Grid (EPSG:5187): Easting={:.3} m, Northing={:.3} m",
        e_work, n_work
    );
    println!(
        "  - Submit Grid (EPSG:5179): Easting={:.3} m, Northing={:.3} m",
        e_submit, n_submit
    );

    println!("\nEngine status: Ready for SfM and Dense Reconstruction.");
}
