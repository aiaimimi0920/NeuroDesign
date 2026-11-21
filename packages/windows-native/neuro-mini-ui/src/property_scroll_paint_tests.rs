//! 只检查自有内存 DIB，确保滚动底色、单位和数值框共享几何，不采集任何桌面像素。
use super::*;
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
fn clipped_viewport_keeps_clean_gaps_and_visible_suffix_at_fractional_scroll() {
    unsafe {
        let (_owner, bar) = crate::tests::fixture();
        crate::prepare(bar.0).unwrap();
        let panel = PropertyPanel::new(bar.0, 125).unwrap();
        panel
            .update(
                1,
                (1..=8)
                    .map(|id| PropertyField {
                        id,
                        label: "不透明度（%）",
                        icon: None,
                        kind: PropertyKind::Number {
                            value: 50,
                            min: 0,
                            max: 100,
                            suffix: "%",
                        },
                    })
                    .collect(),
                true,
            )
            .unwrap();
        let d = &panel.0.data;
        let px = |v| crate::layout::px(v, d.dpi.get());
        panel.set_max_height(px(100)).unwrap();
        let buffer = Buffer::new(d.width.get(), d.height.get());
        for offset in [0.0, px(15) as f64] {
            scroll::set(d, offset).unwrap();
            paint::shell(d, d.viewport.get(), buffer.0).unwrap();
            GdiFlush().ok().unwrap();
            let y = px(29) - offset as i32;
            assert_eq!(
                GetPixel(buffer.0, px(5), y),
                paint::rgb(d.painter.palette.surface)
            );
            let frame = d.controls.borrow()[1].frame;
            let right = frame.right - px(14);
            let base = paint::rgb(crate::paint::mix(
                d.painter.palette.control,
                d.painter.palette.text,
                0.06,
            ));
            let mut ink = 0;
            for y in frame.top + px(4) - offset as i32..frame.bottom - px(4) - offset as i32 {
                for x in right - px(12)..right {
                    if GetPixel(buffer.0, x, y) != base {
                        ink += 1;
                    }
                }
            }
            assert!(
                ink > 0,
                "percent suffix must be drawn independently of EDIT"
            );
        }
    }
}
