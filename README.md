# AGI (Archaeological Geometric Intelligence)

[![CI](https://github.com/kesekikwon-netizen/agi/actions/workflows/ci.yml/badge.svg)](https://github.com/kesekikwon-netizen/agi/actions/workflows/ci.yml)

**AGI**는 고고학 현장 조사 및 유물 실측을 위해 설계된 순수 Rust(100% Pure Rust) 기반의 차세대 3D 사진측량(Photogrammetry) 및 도면화 엔진입니다.

`mac-hgis (Strata)`의 직관적인 **한 줄 리본 UI/UX 철학**을 계승하고, **Agisoft Metashape 및 Bentley iTwin Capture급의 정밀 블록/청크 사진측량 파이프라인**을 제공합니다.

## 🚀 Key Highlights

- **100% Pure Rust Architecture**: 메모리 안전성 보장, 0% 런타임 크래시 지향, 극도로 가벼운 단일 바이너리.
- **국가유산 발굴조사 표준 좌표계 내장**:
  - KGD2002 중부원점 (`EPSG:5186`)
  - KGD2002 동부원점 (`EPSG:5187`)
  - 국가유산청 제출용 통합좌표계/UTM-K (`EPSG:5179`)
  - 로컬 실측 데카르트 좌표계
- **Agisoft & iTwin Capture급 블록 파이프라인**:
  - Pinhole 카메라 모델 및 Brown-Conrady 렌즈 왜곡 보정
  - 번들 조정 (Sparse Bundle Adjustment)
  - 마커/타깃 자동 인식 및 실측 스케일바 캘리브레이션
  - 고밀도 입체 복원(MVS) 및 평면/단면 정사도면 자동화
- **크로스플랫폼 자동 CI**: macOS, Linux, Windows 3대 OS 자동 빌드 및 단위 테스트 검증.

## 🛠️ Quick Start

### Build & Run
```bash
# Workspace Build
cargo build --workspace

# Run Tests
cargo test --workspace

# Run CLI Demo
cargo run -p agi-cli
```

## 📂 Workspace Structure

```
agi/
├── Cargo.toml                  # Workspace Master Manifest
├── .github/
│   └── workflows/
│       └── ci.yml            # Multi-Platform Rust CI Matrix
└── crates/
    ├── agi-core/              # 3D Math, Pinhole Camera, Korean Datum CRS (5186/5187/5179)
    └── agi-cli/               # CLI Inspection & Runner Binary
```

## 📄 Architecture Blueprint

자세한 UI/UX 명세 및 내부 파이프라인 설계는 공식 아키텍처 문서를 참고하십시오:
- [AGI Engine Architecture Blueprint](docs/AGI_ENGINE_ARCHITECTURE_BLUEPRINT.md)

## 📄 License

MIT OR Apache-2.0
