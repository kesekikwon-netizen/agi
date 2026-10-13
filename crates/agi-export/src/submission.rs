use serde::{Deserialize, Serialize};

/// Archaeological investigation metadata required for National Heritage Agency submission.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmissionMetadata {
    pub site_name: String,
    pub trench_id: String,
    pub investigation_agency: String,
    pub surveyor: String,
    pub crs_code: String,
    pub gsd_m: f64,
    pub rms_reprojection_px: f64,
    pub gcp_rms_error_m: f64,
    pub photo_count: usize,
    pub point_count: usize,
}

#[derive(Default)]
pub struct SubmissionPacker;

impl SubmissionPacker {
    pub fn new() -> Self {
        Self
    }

    /// Generates standard MANIFEST.sha256 file content for legal provenance.
    pub fn build_manifest(&self, files: &[(&str, &[u8])]) -> String {
        let mut manifest = String::new();
        for &(filename, bytes) in files {
            let hash = compute_sha256(bytes);
            manifest.push_str(&format!("{}  {}\n", hash, filename));
        }
        manifest
    }

    /// Generates formal Cultural Heritage Agency 3D Photogrammetry Provenance Report.
    pub fn generate_provenance_report(&self, meta: &SubmissionMetadata) -> String {
        format!(
            "# 매장유산 3D 사진측량 및 도면화 조사보고서 (National Heritage Submission)\n\n\
             ## 1. 유적 및 조사 개요\n\
             - **유적명**: {}\n\
             - **조사 트렌치/피트**: {}\n\
             - **조사기관**: {}\n\
             - **책임조사원**: {}\n\
             - **조사 좌표계**: {}\n\n\
             ## 2. 3D 사진측량 복원 정밀도 및 성과\n\
             - **촬영 사진 수**: {} 매\n\
             - **고밀도 복원 점군 수**: {} 점\n\
             - **지상표본거리 (GSD)**: {:.3} mm/px\n\
             - **광속조정 RMS 재투영 오차**: {:.2} px (허용기준: <= 0.50 px)\n\
             - **기준점 (GCP) 잔차 오차**: {:.2} mm (허용기준: <= 5.0 mm)\n\n\
             ## 3. 데이터 무결성 검증\n\
             본 성과물에 포함된 모든 원시 사진, 3D 점군(PLY), AutoCAD 도면(DXF), 정사영상(GeoTIFF)은 \
             `MANIFEST.sha256` 파일에 기록된 암호학적 해시값에 의해 무결성이 보증됩니다.\n",
            meta.site_name,
            meta.trench_id,
            meta.investigation_agency,
            meta.surveyor,
            meta.crs_code,
            meta.photo_count,
            format_integer_with_commas(meta.point_count),
            meta.gsd_m * 1000.0,
            meta.rms_reprojection_px,
            meta.gcp_rms_error_m * 1000.0
        )
    }
}

fn format_integer_with_commas(n: usize) -> String {
    let s = n.to_string();
    let mut result = String::new();
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len();

    for (i, &ch) in chars.iter().enumerate() {
        result.push(ch);
        let remaining = len - 1 - i;
        if remaining > 0 && remaining.is_multiple_of(3) {
            result.push(',');
        }
    }
    result
}

/// Computes NIST FIPS 180-4 SHA-256 hash in pure, memory-safe Rust.
pub fn compute_sha256(data: &[u8]) -> String {
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];

    let mut h0: u32 = 0x6a09e667;
    let mut h1: u32 = 0xbb67ae85;
    let mut h2: u32 = 0x3c6ef372;
    let mut h3: u32 = 0xa54ff53a;
    let mut h4: u32 = 0x510e527f;
    let mut h5: u32 = 0x9b05688c;
    let mut h6: u32 = 0x1f83d9ab;
    let mut h7: u32 = 0x5be0cd19;

    let bit_len = (data.len() as u64) * 8;

    // Pre-processing (Padding)
    let mut msg = data.to_vec();
    msg.push(0x80);
    while (msg.len() % 64) != 56 {
        msg.push(0x00);
    }
    msg.extend_from_slice(&bit_len.to_be_bytes());

    // Process blocks
    for chunk in msg.chunks_exact(64) {
        let mut w = [0u32; 64];
        for i in 0..16 {
            w[i] = u32::from_be_bytes([
                chunk[i * 4],
                chunk[i * 4 + 1],
                chunk[i * 4 + 2],
                chunk[i * 4 + 3],
            ]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }

        let mut a = h0;
        let mut b = h1;
        let mut c = h2;
        let mut d = h3;
        let mut e = h4;
        let mut f = h5;
        let mut g = h6;
        let mut h = h7;

        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let temp1 = h
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(maj);

            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }

        h0 = h0.wrapping_add(a);
        h1 = h1.wrapping_add(b);
        h2 = h2.wrapping_add(c);
        h3 = h3.wrapping_add(d);
        h4 = h4.wrapping_add(e);
        h5 = h5.wrapping_add(f);
        h6 = h6.wrapping_add(g);
        h7 = h7.wrapping_add(h);
    }

    format!(
        "{:08x}{:08x}{:08x}{:08x}{:08x}{:08x}{:08x}{:08x}",
        h0, h1, h2, h3, h4, h5, h6, h7
    )
}
