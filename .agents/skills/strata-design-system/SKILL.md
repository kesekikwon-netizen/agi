---
name: strata-design-system
description: Enforce Strata (mac-hgis) exact UI/UX design tokens, layout principles, and field-legible styling for AGI photogrammetry software.
---

# Strata (mac-hgis) UI/UX Design System Guide

When designing, updating, or building any UI/UX component for `agi`, you MUST strictly adhere to the exact design language created by the designer of `Strata (mac-hgis)`.

## 1. Core Visual Principles
- **No Dark Sci-Fi Themes**: Strata is designed for bright daylight archaeological field tents and excavation monitors. The base theme is **clean, high-contrast white and slate-navy**.
- **One-Row Ribbon (한 줄 리본)**: There is NO traditional multi-level menu bar. All primary actions exist on a single-row ribbon toolbar.
- **Explicit Save Only**: No auto-save. Save only happens on `Ctrl+S`. Unsaved changes show ` *` in the window title.
- **Startup Rule**: The app opens to the Home screen only. It never automatically restores or opens heavy projects on launch.

## 2. Design Tokens (Exact Hex Colors)
- `surface`: `#FFFFFF` (Card grounds, panels, clean background)
- `desk`: `#F4F6F8` (Page background under cards and viewports)
- `rail`: `#0B3A63` (Strata Navy - header bar and ribbon accent)
- `accent`: `#1F6FB2` (Strata Blue - primary action button)
- `accentHover`: `#185E99`
- `accentDeep`: `#12558D`
- `accentWash`: `#E3EEF8` (Soft blue wash for chosen/selected tool chips)
- `ink`: `#1D2733` (Deep slate body text, high contrast >= 14:1)
- `inkMuted`: `#5B6875` (Secondary labels, shortcuts)
- `border`: `#DCE3EA` (Subtle divider lines)
- `borderStrong`: `#B8C4CF` (Input boundaries)
- `ok`: `#256B42` (Success / aligned status)
- `danger`: `#B42318` (Error / uncalibrated warnings)
- `warn`: `#7A4A00` on `#FFF4DB` (Archaeological amber chip)

## 3. Typography
- Windows / macOS standard: **Malgun Gothic (맑은 고딕)** or **IBM Plex Sans KR**.
- Body: 11px - 13px (crisp, high-density archaeological data tables).
- Headers / Ribbon titles: 12px - 14px bold.

## 4. Keyboard Shortcuts
- `Ctrl+N` (새 조사), `Ctrl+O` (조사 열기), `Ctrl+S` (저장)
- `Ctrl+I` (사진 추가), `Ctrl+R` (사진 정렬), `Ctrl+T` (타깃 인식), `Ctrl+K` (스케일 지정)
- `Ctrl+Alt+O` (평면 정사영상), `Ctrl+Alt+S` (토층 단면도)
- `Ctrl+E` (검수·제출)
- `F9` (작업 트리 패널 접기/펴기)
- `Ctrl+F11` (뷰포트 넓게 보기)
