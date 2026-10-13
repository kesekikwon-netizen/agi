use agi_core::CoordinateSystem;
use agi_ortho::Orthomosaic;

/// ESRI World File (`.tfw`) definition for georeferencing raster images.
#[derive(Debug, Clone)]
pub struct WorldFile {
    pub x_pixel_size: f64,
    pub rotation_y: f64,
    pub rotation_x: f64,
    pub y_pixel_size: f64,
    pub center_ul_x: f64,
    pub center_ul_y: f64,
}

impl WorldFile {
    pub fn from_ortho(ortho: &Orthomosaic) -> Self {
        Self {
            x_pixel_size: ortho.gsd_m,
            rotation_y: 0.0,
            rotation_x: 0.0,
            y_pixel_size: -ortho.gsd_m,
            center_ul_x: ortho.min_x + ortho.gsd_m * 0.5,
            center_ul_y: ortho.max_y - ortho.gsd_m * 0.5,
        }
    }

    #[allow(clippy::inherent_to_string)]
    pub fn to_string(&self) -> String {
        format!(
            "{:.8}\n{:.8}\n{:.8}\n{:.8}\n{:.8}\n{:.8}\n",
            self.x_pixel_size,
            self.rotation_y,
            self.rotation_x,
            self.y_pixel_size,
            self.center_ul_x,
            self.center_ul_y
        )
    }
}

pub struct GeoTiffWriter;

impl GeoTiffWriter {
    /// Serializes an orthomosaic into a pure Rust uncompressed GeoTIFF binary stream.
    pub fn write_ortho_tiff(ortho: &Orthomosaic, _crs: CoordinateSystem) -> Option<Vec<u8>> {
        let width = ortho.width as u32;
        let height = ortho.height as u32;

        let num_pixels = (width * height) as usize;
        let rgb_bytes_len = num_pixels * 3;

        let mut out = Vec::new();

        // 1. TIFF Header (8 bytes): "II" (little-endian), magic 42, IFD offset = 8
        out.extend_from_slice(b"II\x2A\x00");
        let ifd_offset: u32 = 8;
        out.extend_from_slice(&ifd_offset.to_le_bytes());

        // 2. IFD (Image File Directory)
        // Tags:
        // 256: ImageWidth (LONG)
        // 257: ImageLength (LONG)
        // 258: BitsPerSample (SHORT, count 3 -> offset to values)
        // 259: Compression (SHORT, 1 = None)
        // 262: PhotometricInterpretation (SHORT, 2 = RGB)
        // 273: StripOffsets (LONG -> offset to raw RGB bytes)
        // 277: SamplesPerPixel (SHORT, 3)
        // 278: RowsPerStrip (LONG = height)
        // 279: StripByteCounts (LONG = rgb_bytes_len)
        // 33550: ModelPixelScaleTag (DOUBLE, count 3 -> [gsd, gsd, 0.0])
        // 33922: ModelTiepointTag (DOUBLE, count 6 -> [0, 0, 0, min_x, max_y, 0])

        let num_entries: u16 = 11;
        out.extend_from_slice(&num_entries.to_le_bytes());

        // We calculate offsets for extra data:
        // Size of IFD = 2 + 11 * 12 + 4 (next IFD) = 138 bytes.
        // IFD starts at 8, so IFD ends at 8 + 138 = 146.
        let mut extra_offset: u32 = 146;

        let bits_per_sample_offset = extra_offset;
        extra_offset += 3 * 2; // 6 bytes for 3 SHORTs

        let pixel_scale_offset = extra_offset;
        extra_offset += 3 * 8; // 24 bytes for 3 DOUBLEs

        let tiepoint_offset = extra_offset;
        extra_offset += 6 * 8; // 48 bytes for 6 DOUBLEs

        // Align image data to 4-byte boundary
        let image_data_offset = (extra_offset + 3) & !3;

        // Helper to write an IFD directory entry (12 bytes)
        let mut write_entry = |tag: u16, dtype: u16, count: u32, val_or_offset: u32| {
            out.extend_from_slice(&tag.to_le_bytes());
            out.extend_from_slice(&dtype.to_le_bytes());
            out.extend_from_slice(&count.to_le_bytes());
            out.extend_from_slice(&val_or_offset.to_le_bytes());
        };

        // Type 3 = SHORT, Type 4 = LONG, Type 12 = DOUBLE
        write_entry(256, 4, 1, width);
        write_entry(257, 4, 1, height);
        write_entry(258, 3, 3, bits_per_sample_offset);
        write_entry(259, 3, 1, 1);
        write_entry(262, 3, 1, 2);
        write_entry(273, 4, 1, image_data_offset);
        write_entry(277, 3, 1, 3);
        write_entry(278, 4, 1, height);
        write_entry(279, 4, 1, rgb_bytes_len as u32);
        write_entry(33550, 12, 3, pixel_scale_offset);
        write_entry(33922, 12, 6, tiepoint_offset);

        // Next IFD offset = 0
        out.extend_from_slice(&0u32.to_le_bytes());

        // 3. Write extra data values
        // BitsPerSample: [8, 8, 8]
        out.extend_from_slice(&8u16.to_le_bytes());
        out.extend_from_slice(&8u16.to_le_bytes());
        out.extend_from_slice(&8u16.to_le_bytes());

        // ModelPixelScaleTag: [gsd, gsd, 0.0]
        out.extend_from_slice(&ortho.gsd_m.to_le_bytes());
        out.extend_from_slice(&ortho.gsd_m.to_le_bytes());
        out.extend_from_slice(&0.0f64.to_le_bytes());

        // ModelTiepointTag: [0.0, 0.0, 0.0, min_x, max_y, 0.0]
        out.extend_from_slice(&0.0f64.to_le_bytes());
        out.extend_from_slice(&0.0f64.to_le_bytes());
        out.extend_from_slice(&0.0f64.to_le_bytes());
        out.extend_from_slice(&ortho.min_x.to_le_bytes());
        out.extend_from_slice(&ortho.max_y.to_le_bytes());
        out.extend_from_slice(&0.0f64.to_le_bytes());

        // Pad until image_data_offset
        while out.len() < image_data_offset as usize {
            out.push(0);
        }

        // 4. Write RGB raster pixels
        for pixel in &ortho.rgb_data {
            out.push(pixel[0]);
            out.push(pixel[1]);
            out.push(pixel[2]);
        }

        Some(out)
    }
}
