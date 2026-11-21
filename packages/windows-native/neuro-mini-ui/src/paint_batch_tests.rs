//! 真实内存 DC 验证完整帧提交及标准 child 的原位打印；不读取用户屏幕。
use super::*;
use crate::{properties::*, tests::fixture};
use windows::core::Error;

struct Buffer {
    dc: HDC,
    bitmap: HBITMAP,
    old: HGDIOBJ,
}
impl Buffer {
    unsafe fn new(width: i32, height: i32) -> Self {
        let dc = CreateCompatibleDC(None);
        let mut bits = std::ptr::null_mut();
        let bitmap = CreateDIBSection(
            None,
            &BITMAPINFO {
                bmiHeader: BITMAPINFOHEADER {
                    biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                    biWidth: width,
                    biHeight: -height,
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
        assert!(!dc.is_invalid());
        let old = SelectObject(dc, HGDIOBJ(bitmap.0));
        Self { dc, bitmap, old }
    }
}
impl Drop for Buffer {
    fn drop(&mut self) {
        unsafe {
            SelectObject(self.dc, self.old);
            DeleteObject(HGDIOBJ(self.bitmap.0)).ok().unwrap();
            DeleteDC(self.dc).ok().unwrap();
        }
    }
}
unsafe fn fill(dc: HDC, rect: RECT, colour: u32) {
    assert_ne!(
        FillRect(dc, &rect, HBRUSH(background(dc, colour).0 as *mut _)),
        0
    );
}

#[test]
fn partial_and_failed_draws_leave_destination_unchanged_until_complete_frame_commit() {
    unsafe {
        let destination = Buffer::new(32, 32);
        let bounds = RECT {
            left: 0,
            top: 0,
            right: 32,
            bottom: 32,
        };
        fill(destination.dc, bounds, 0xff0000);
        let red = GetPixel(destination.dc, 16, 16);
        let error = frame(destination.dc, bounds, |out| {
            fill(out, bounds, 0x0000ff);
            assert_eq!(GetPixel(destination.dc, 16, 16), red);
            Err(Error::from_hresult(E_FAIL))
        })
        .unwrap_err();
        assert_eq!(error.code(), E_FAIL);
        assert_eq!(
            GetPixel(destination.dc, 16, 16),
            red,
            "failed preparation must not publish a cleared frame"
        );
        frame(destination.dc, bounds, |out| {
            fill(out, bounds, 0x0000ff);
            assert_eq!(
                GetPixel(destination.dc, 16, 16),
                red,
                "intermediate D2D/GDI work is still private"
            );
            Ok(())
        })
        .unwrap();
        assert_eq!(GetPixel(destination.dc, 16, 16), COLORREF(0xff0000));
    }
}

#[test]
fn whole_toolbar_frame_contains_actual_owner_draw_children_in_their_final_slots() {
    unsafe {
        let (_owner, bar) = fixture();
        crate::prepare(bar.0).unwrap();
        crate::update(bar.0, 1, crate::Icon::stroke(""), true, true, true, "选择").unwrap();
        let mut bounds = RECT::default();
        GetClientRect(bar.0, &mut bounds).unwrap();
        let target = Buffer::new(bounds.right, bounds.bottom);
        window(bar.0, target.dc, |dc| {
            fill(dc, bounds, 0x090c11);
            Ok(())
        })
        .unwrap();
        let mut origin = POINT::default();
        MapWindowPoints(
            Some(GetDlgItem(Some(bar.0), 1).unwrap()),
            Some(bar.0),
            std::slice::from_mut(&mut origin),
        );
        let palette = crate::Palette::default();
        let colour = crate::paint::mix(
            crate::paint::mix(palette.control, palette.text, 0.06),
            palette.signal,
            0.12,
        );
        assert_eq!(
            GetPixel(target.dc, origin.x + 14, origin.y + 24),
            COLORREF(((colour & 255) << 16) | (colour & 0xff00) | ((colour >> 16) & 255))
        );
    }
}

#[test]
fn whole_property_frame_contains_real_child_colour_and_preserves_capture_affinity() {
    unsafe {
        let (_owner, bar) = fixture();
        crate::prepare(bar.0).unwrap();
        set_properties(
            bar.0,
            1,
            vec![PropertyField {
                id: 1,
                label: "颜色",
                icon: None,
                kind: PropertyKind::Color {
                    rgb: Some(0xd92211),
                    stroke: false,
                },
            }],
            true,
        )
        .unwrap();
        let popup = property_window(bar.0).unwrap();
        SetWindowDisplayAffinity(popup, WDA_EXCLUDEFROMCAPTURE).unwrap();
        let mut affinity = 0;
        GetWindowDisplayAffinity(popup, &mut affinity).unwrap();
        assert_eq!(affinity, WDA_EXCLUDEFROMCAPTURE.0);
        let mut bounds = RECT::default();
        GetClientRect(popup, &mut bounds).unwrap();
        let target = Buffer::new(bounds.right, bounds.bottom);
        window(popup, target.dc, |dc| {
            fill(dc, bounds, 0x090c11);
            Ok(())
        })
        .unwrap();
        assert_eq!(GetPixel(target.dc, 19, 19), COLORREF(0x1122d9));
    }
}

#[test]
fn redraw_hold_restores_visible_hosts_on_error_but_never_reveals_hidden_hosts() {
    unsafe {
        let (owner, bar) = fixture();
        crate::prepare(bar.0).unwrap();
        drop(Hold::new(bar.0));
        assert!(!IsWindowVisible(bar.0).as_bool());
        let _ = ShowWindow(owner.0, SW_SHOWNOACTIVATE);
        let _ = ShowWindow(bar.0, SW_SHOWNOACTIVATE);
        let fail = || -> Result<()> {
            let _hold = Hold::new(bar.0);
            assert!(!IsWindowVisible(bar.0).as_bool());
            Err(Error::from_hresult(E_FAIL))
        };
        assert_eq!(fail().unwrap_err().code(), E_FAIL);
        assert!(IsWindowVisible(bar.0).as_bool());
        assert!(!GetUpdateRect(bar.0, None, false).as_bool());
        let hold = Hold::new(bar.0);
        DestroyWindow(owner.0).unwrap();
        drop(hold);
        assert!(!IsWindow(Some(bar.0)).as_bool());
    }
}
