# CLAUDE.md

# AGI (Archaeological Geometric Intelligence) 개발 지침

사용자는 개발자가 아닌 고고학 현장 전문가다. 본 프로젝트는 `mac-hgis (Strata)`의 디자이너가 설계한 UI/UX 디자인 시스템과 Agisoft Metashape 및 Bentley iTwin Capture급의 3D 사진측량 엔진을 순수 Rust(100% Pure Rust)로 구현하는 독립 소프트웨어다.

---

## 1. 일하는 방식 및 슈퍼파워 스킬(Superpowers) 연동 규칙

1. **단계별 슈퍼파워 스킬 적극 활용**:
   - 코드 수정 및 신규 알고리즘 작성 시: `superpowers:test-driven-development` (시험 먼저 작성 후 구현)
   - 버그, 오차, 테스트 실패 발생 시: `superpowers:systematic-debugging` (원인 분석 후 자체 수정)
   - 작업 완료 보고 전: `superpowers:verification-before-completion` (반드시 `cargo test --workspace` 실행 증거 확인)
   - 세부 구현 계획 실행 시: `superpowers:executing-plans`
2. **자율적 주도성 (Autonomous Proactivity)**:
   - 비개발자 사용자에게 빌드, 테스트, 컴파일 실행을 떠넘기지 마라.
   - 모든 수정 후 스스로 `cargo test --workspace`를 실행하여 100% 통과를 확인하라.
   - 컴파일 에러나 테스트 실패 시 자율적으로 원인을 분석하고 스스로 수정한 뒤(Self-Correction) 보고하라.
3. **설명 방식**:
   - 비개발자의 눈높이에 맞추어 쉬운 일상 언어와 실무적 관점으로 간결하게 설명하라. 난해한 문법 용어 나열 금지.

---

## 2. Strata (mac-hgis) 디자인 시스템 준수 규칙 (절대 위반 금지)

1. **디자인 테마 (No Dark Sci-Fi Theme)**:
   - 본 프로그램은 야외 발굴 현장 천막과 강한 햇빛 아래의 야외 노트북 모니터에서 높은 시인성을 보장해야 한다.
   - 임의의 어두운 SF 게임풍 다크 테마를 금지하며, Strata의 정품 테마인 **화이트 바탕(`surface`: `#FFFFFF`), 라이트 그레이 데스크(`desk`: `#F4F6F8`), 네이비 프레임(`rail`: `#0B3A63`), 블루 악센트(`accent`: `#1F6FB2`)**를 기본으로 한다.
2. **단일 한 줄 리본 막대 (One-Row Ribbon Bar)**:
   - 다단계 풀다운 메뉴 막대를 사용하지 않는다.
   - 상단은 단일 한 줄 리본(`조사` · `사진·정렬` · `3D 복원` · `도면화` · `내보내기` · `기타 › 더보기`)으로 구성한다.
3. **명시적 저장 (Explicit Save Only)**:
   - 자동 저장은 없으며, 오직 `Ctrl+S`로만 저장한다.
   - 저장되지 않은 변경사항이 있으면 창 제목 끝에 ` *`를 표시한다.
4. **시작 화면 규칙**:
   - 앱 구동 시 무거운 이전 프로젝트를 자동 복원하지 않고, 반드시 **홈 화면**만 연다.
5. **글꼴 (Typography)**:
   - 기본 글꼴은 11~13px에서 가독성이 가장 뛰어난 **맑은 고딕(Malgun Gothic)** 또는 번들된 **IBM Plex Sans KR**을 사용한다.
6. **단축키 체계**:
   - `Ctrl+N`(새 조사), `Ctrl+O`(열기), `Ctrl+S`(저장), `Ctrl+I`(사진 추가), `Ctrl+R`(사진 정렬), `Ctrl+T`(타깃 인식), `Ctrl+K`(스케일), `Ctrl+Alt+O`(평면 정사영상), `Ctrl+Alt+S`(토층 단면도), `Ctrl+E`(검수·제출), `F9`(작업 트리 접기), `Ctrl+F11`(뷰포트 넓게 보기).

---

## 3. Rust 사진측량 엔진 기술 표준

1. **100% Pure Rust & 메모리 안전성**:
   - 런타임 크래시 0%를 지향하며, 안전하지 않은(`unsafe`) 메모리 조작을 엄격히 배제한다.
2. **좌표계 및 측지학 표준**:
   - KGD2002 중부원점(`EPSG:5186`), 동부원점(`EPSG:5187`), 국가유산청 제출용 UTM-K(`EPSG:5179`) 간의 밀리미터 단위 양방향 변환을 보장한다.
3. **검수·제출 파이프라인**:
   - 제출 성과품은 `EPSG:5179` 변환 SHP, 1:1 정사도면 PDF, 무손실 GeoTIFF, `MANIFEST.sha256` 파일 세트로 구성된다.

---

## 4. 빌드, 테스트 및 검증 명령어

- 전체 빌드: `cargo build --workspace`
- 전체 단위 테스트: `cargo test --workspace`
- CLI 실행 검증: `cargo run -p agi-cli`
- 코드 포맷 검사: `cargo fmt --check`
- 린트 검사: `cargo clippy --workspace -- -D warnings`
