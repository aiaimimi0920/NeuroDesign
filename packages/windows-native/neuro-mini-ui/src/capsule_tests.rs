//! 几何与内存 DIB 验证；不采集或操作用户桌面。
use super::*;
#[test]
fn output_action_keeps_the_baseline_image_side_inset_instead_of_mirroring_the_input_slot() {
    for dpi in [96, 120, 144, 192] {
        let row = CapsuleLayout::new(
            (300, 40),
            dpi,
            CapsuleSide::Output,
            true,
            CapsuleSize::Normal,
        );
        assert_eq!(row.inline_action(dpi).unwrap().x, row.body.x + px(18, dpi));
        let label = row.label(dpi, true).unwrap();
        assert_eq!(label.x, row.body.x + px(44, dpi));
        assert_eq!(label.width, px(72, dpi));
        assert!(label.x > row.inline_action(dpi).unwrap().x + px(16, dpi));
    }
}
#[test]
fn continuous_backdrop_fills_row_gaps_with_round_outer_corners_and_respects_host_clipping() {
    unsafe {
        let buffer = Buffer::new(600, 160);
        let mut painter = CapsulePainter::new().unwrap();
        let frame = crate::Frame {
            x: 20,
            y: 20,
            width: 256,
            height: 100,
        };
        let saved = SaveDC(buffer.0);
        ExcludeClipRect(buffer.0, 120, 40, 160, 80);
        painter.backdrop(buffer.0, frame, 96).unwrap();
        RestoreDC(buffer.0, saved).ok().unwrap();
        GdiFlush().ok().unwrap();
        assert_eq!(GetPixel(buffer.0, 80, 52), COLORREF(0x120d0a));
        assert_eq!(GetPixel(buffer.0, 20, 20), COLORREF(0));
        assert_eq!(GetPixel(buffer.0, 140, 52), COLORREF(0));
        painter
            .backdrop(
                buffer.0,
                crate::Frame {
                    x: 80,
                    y: 52,
                    ..frame
                },
                96,
            )
            .unwrap();
        GdiFlush().ok().unwrap();
        assert_eq!(
            GetPixel(buffer.0, 80, 52),
            COLORREF(0x120d0a),
            "overlapping rounded corners must retain the earlier group"
        );
    }
}
#[test]
fn expanded_capsules_keep_the_image_side_edge_and_move_the_connection_outwards() {
    for dpi in [96, 120, 144, 192, 240] {
        for side in [CapsuleSide::Input, CapsuleSide::Output] {
            let closed = CapsuleLayout::new((-400, -200), dpi, side, false, CapsuleSize::Normal);
            let open = CapsuleLayout::new((-400, -200), dpi, side, true, CapsuleSize::Normal);
            assert_eq!(
                (closed.frame.width, open.frame.width, open.frame.height),
                (px(28, dpi), px(256, dpi), px(28, dpi))
            );
            assert_eq!(closed.connection(), (-400, -200));
            let delta = px(256, dpi) - px(28, dpi);
            assert_eq!(
                open.connection().0,
                -400 + if side == CapsuleSide::Input {
                    -delta
                } else {
                    delta
                }
            );
            assert_eq!(open.connection().1, -200);
            assert_eq!(closed.frame.y, open.frame.y);
            if side == CapsuleSide::Input {
                assert_eq!(
                    closed.frame.x + closed.frame.width,
                    open.frame.x + open.frame.width
                );
            } else {
                assert_eq!(closed.frame.x, open.frame.x);
            }
        }
    }
}
#[test]
fn compact_markers_never_expand_into_full_rows() {
    for side in [CapsuleSide::Input, CapsuleSide::Output] {
        let compact = CapsuleLayout::new((20, 30), 192, side, true, CapsuleSize::Compact);
        assert!(!compact.expanded);
        assert_eq!(compact.connection(), (20, 30));
        assert_eq!((compact.frame.width, compact.frame.height), (16, 16));
    }
}
struct Buffer(HDC, HBITMAP, HGDIOBJ);
impl Buffer {
    unsafe fn new(width: i32, height: i32) -> Self {
        let dc = CreateCompatibleDC(None);
        assert!(!dc.is_invalid());
        let mut bits = std::ptr::null_mut();
        let info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width,
                biHeight: -height,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };
        let bitmap = CreateDIBSection(Some(dc), &info, DIB_RGB_COLORS, &mut bits, None, 0).unwrap();
        let old = SelectObject(dc, HGDIOBJ(bitmap.0));
        Self(dc, bitmap, old)
    }
}
impl Drop for Buffer {
    fn drop(&mut self) {
        unsafe {
            SelectObject(self.0, self.2);
            let _ = DeleteObject(HGDIOBJ(self.1 .0));
            let _ = DeleteDC(self.0);
        }
    }
}
#[test]
fn drawing_uses_original_icon_colors_and_clips_only_the_collapsed_image_side_half() {
    unsafe {
        let buffer = Buffer::new(600, 100);
        let mut painter = CapsulePainter::new().unwrap();
        let icon = Icon::svg(
            r##"<svg viewBox="0 0 24 24" fill="#06b6d4"><rect x="0" y="0" width="24" height="24" /></svg>"##,
        );
        for side in [CapsuleSide::Input, CapsuleSide::Output] {
            for expanded in [false, true] {
                let layout = CapsuleLayout::new((300, 40), 96, side, expanded, CapsuleSize::Normal);
                painter
                    .paint(
                        buffer.0,
                        layout,
                        96,
                        CapsuleAppearance {
                            icon,
                            label: "名称",
                            enabled: true,
                            focused: false,
                            action: None,
                            background: 0,
                        },
                    )
                    .unwrap();
                GdiFlush().ok().unwrap();
                let (x, y) = layout.connection();
                let blue = COLORREF(0xd4b606);
                if expanded {
                    assert_eq!(GetPixel(buffer.0, x - 6, y), blue);
                    assert_eq!(GetPixel(buffer.0, x + 6, y), blue);
                    assert_eq!(
                        GetPixel(buffer.0, layout.frame.x + 1, layout.frame.y + 1),
                        COLORREF(0)
                    );
                } else if side == CapsuleSide::Input {
                    assert_eq!(GetPixel(buffer.0, x - 6, y), blue);
                    assert_eq!(GetPixel(buffer.0, x + 6, y), COLORREF(0));
                } else {
                    assert_eq!(GetPixel(buffer.0, x - 6, y), COLORREF(0));
                    assert_eq!(GetPixel(buffer.0, x + 6, y), blue);
                }
            }
        }
    }
}
#[test]
fn inline_action_stays_between_label_and_value_without_entering_anchor_or_collapsed_hit_area() {
    for dpi in [96, 120, 144, 192] {
        for side in [CapsuleSide::Input, CapsuleSide::Output] {
            for size in [CapsuleSize::Normal, CapsuleSize::Compact] {
                let closed = CapsuleLayout::new((-200, 100), dpi, side, false, size);
                assert!(closed.inline_action(dpi).is_none());
                let open = CapsuleLayout::new((-200, 100), dpi, side, true, size);
                if size == CapsuleSize::Compact {
                    assert!(open.inline_action(dpi).is_none());
                    continue;
                }
                let f = open.inline_action(dpi).unwrap();
                assert_eq!((f.width, f.height), (px(16, dpi), px(16, dpi)));
                assert!(f.x >= open.body.x && f.x + f.width <= open.body.x + open.body.width);
                assert!(f.y >= open.body.y && f.y + f.height <= open.body.y + open.body.height);
                assert!(!(f.x..f.x + f.width).contains(&open.connection().0));
            }
        }
    }
}
#[test]
fn inline_action_is_actually_drawn_with_its_own_state_and_does_not_overpaint_the_anchor() {
    unsafe {
        let buffer = Buffer::new(600, 100);
        let mut painter = CapsulePainter::new().unwrap();
        let icon = Icon::svg(
            r#"<svg viewBox="0 0 24 24" fill="currentColor"><rect x="0" y="0" width="24" height="24" /></svg>"#,
        );
        for side in [CapsuleSide::Input, CapsuleSide::Output] {
            let layout = CapsuleLayout::new((300, 40), 96, side, true, CapsuleSize::Normal);
            let frame = layout.inline_action(96).unwrap();
            let mut colors = Vec::new();
            let mut anchors = Vec::new();
            for active in [false, true] {
                painter
                    .paint(
                        buffer.0,
                        layout,
                        96,
                        CapsuleAppearance {
                            icon,
                            label: "很长的端口名称不能覆盖行内按钮",
                            enabled: true,
                            focused: false,
                            action: Some(CapsuleAction {
                                icon,
                                active,
                                pressed: false,
                            }),
                            background: 0,
                        },
                    )
                    .unwrap();
                GdiFlush().ok().unwrap();
                colors.push(GetPixel(
                    buffer.0,
                    frame.x + frame.width / 2,
                    frame.y + frame.height / 2,
                ));
                let (x, y) = layout.connection();
                anchors.push(GetPixel(buffer.0, x, y));
            }
            assert_ne!(colors[0], colors[1]);
            assert_eq!(anchors[0], anchors[1]);
        }
    }
}
