//! 接边的 region 与离屏像素共用几何；不读取用户屏幕。
use super::*;
use crate::{paint::Painter, tests::fixture};
#[test]
fn joined_corners_are_square_but_outer_corners_remain_round_at_every_dpi() {
    unsafe {
        let (_owner, bar) = fixture();
        for dpi in [96, 120, 144, 192] {
            let (w, h) = (crate::layout::px(200, dpi), crate::layout::px(38, dpi));
            let a = RECT {
                left: 0,
                top: 100,
                right: w,
                bottom: 100 + h,
            };
            let b = RECT {
                left: 0,
                top: 100 - h,
                right: w,
                bottom: 100,
            };
            region(bar.0, w, h, dpi, Join::between(a, b)).unwrap();
            let r = Region(CreateRectRgn(0, 0, 0, 0));
            assert_ne!(GetWindowRgn(bar.0, r.0), RGN_ERROR);
            assert!(PtInRegion(r.0, 0, 0).as_bool());
            assert!(PtInRegion(r.0, w - 1, 0).as_bool());
            assert!(!PtInRegion(r.0, 0, h - 1).as_bool());
            region(bar.0, w, h, dpi, Join::default()).unwrap();
            assert_ne!(GetWindowRgn(bar.0, r.0), RGN_ERROR);
            assert!(!PtInRegion(r.0, 0, 0).as_bool());
        }
    }
}
#[test]
fn joined_edge_has_surface_pixels_not_a_duplicate_separator_border() {
    unsafe {
        let p = Painter::new().unwrap();
        let dc = CreateCompatibleDC(None);
        let mut bits = std::ptr::null_mut();
        let bitmap = CreateDIBSection(
            None,
            &BITMAPINFO {
                bmiHeader: BITMAPINFOHEADER {
                    biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                    biWidth: 200,
                    biHeight: -38,
                    biPlanes: 1,
                    biBitCount: 32,
                    ..Default::default()
                },
                ..Default::default()
            },
            DIB_RGB_COLORS,
            &mut bits,
            None,
            0,
        )
        .unwrap();
        let old = SelectObject(dc, HGDIOBJ(bitmap.0));
        p.shell(dc, 200, 38, 96, &[], Join::default()).unwrap();
        let border = GetPixel(dc, 100, 0);
        let a = RECT {
            left: 0,
            top: 38,
            right: 200,
            bottom: 76,
        };
        let b = RECT {
            left: 0,
            top: 0,
            right: 200,
            bottom: 38,
        };
        p.shell(dc, 200, 38, 96, &[], Join::between(a, b)).unwrap();
        assert_eq!(GetPixel(dc, 100, 0), COLORREF(0x110c09));
        assert_ne!(border, COLORREF(0x110c09));
        SelectObject(dc, old);
        DeleteObject(HGDIOBJ(bitmap.0)).ok().unwrap();
        DeleteDC(dc).ok().unwrap();
    }
}
