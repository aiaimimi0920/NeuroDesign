//! 主条的系统擦底与整棵控件树提交；真实 HWND 不等同普通桌面视觉验收。
use super::*;
use windows::Win32::{Foundation::*, Graphics::Gdi::*, UI::WindowsAndMessaging::*};

#[test]
fn toolbar_button_erase_preserves_the_last_complete_frame() {
    unsafe {
        let (_owner, bar) = crate::tests::fixture();
        prepare(bar.0).unwrap();
        let child = GetDlgItem(Some(bar.0), 1).unwrap();
        let screen = GetDC(Some(child));
        let dc = CreateCompatibleDC(Some(screen));
        let bitmap = CreateCompatibleBitmap(screen, 64, 64);
        assert!(!dc.is_invalid() && !bitmap.is_invalid());
        let old = SelectObject(dc, HGDIOBJ(bitmap.0));
        let sentinel = COLORREF(0x123456);
        SetPixel(dc, 10, 10, sentinel);
        SendMessageW(child, WM_ERASEBKGND, Some(WPARAM(dc.0 as usize)), None);
        let pixel = GetPixel(dc, 10, 10);
        SelectObject(dc, old);
        DeleteObject(HGDIOBJ(bitmap.0)).ok().unwrap();
        DeleteDC(dc).ok().unwrap();
        ReleaseDC(Some(child), screen);
        assert_eq!(
            pixel, sentinel,
            "erase published an empty button before WM_DRAWITEM"
        );
    }
}

#[test]
fn toolbar_button_system_erase_uses_the_same_dark_surface_as_the_owner_draw() {
    unsafe {
        let (_owner, bar) = crate::tests::fixture();
        prepare(bar.0).unwrap();
        let screen = GetDC(Some(bar.0));
        let dc = CreateCompatibleDC(Some(screen));
        let bitmap = CreateCompatibleBitmap(screen, 2, 2);
        assert!(!dc.is_invalid() && !bitmap.is_invalid());
        let old = SelectObject(dc, HGDIOBJ(bitmap.0));
        let brush = SendMessageW(
            bar.0,
            WM_CTLCOLORBTN,
            Some(WPARAM(dc.0 as usize)),
            Some(LPARAM(GetDlgItem(Some(bar.0), 1).unwrap().0 as isize)),
        );
        assert_ne!(brush.0, 0);
        assert_ne!(
            FillRect(
                dc,
                &RECT {
                    left: 0,
                    top: 0,
                    right: 2,
                    bottom: 2
                },
                HBRUSH(brush.0 as *mut _)
            ),
            0
        );
        let pixel = GetPixel(dc, 0, 0);
        SelectObject(dc, old);
        DeleteObject(HGDIOBJ(bitmap.0)).ok().unwrap();
        DeleteDC(dc).ok().unwrap();
        ReleaseDC(Some(bar.0), screen);
        assert_eq!(
            pixel,
            COLORREF(0x110c09),
            "standard BUTTON erase must not publish white"
        );
    }
}

#[test]
fn toolbar_completes_its_first_visible_paint_and_keeps_capture_exclusion_available() {
    unsafe {
        let (owner, bar) = crate::tests::fixture();
        prepare(bar.0).unwrap();
        SetWindowDisplayAffinity(bar.0, WDA_EXCLUDEFROMCAPTURE).unwrap();
        SetWindowDisplayAffinity(bar.0, WDA_NONE).unwrap();
        let _ = ShowWindow(owner.0, SW_SHOWNOACTIVATE);
        for _ in 0..3 {
            let _ = ShowWindow(bar.0, SW_SHOWNOACTIVATE);
            assert!(!GetUpdateRect(bar.0, None, false).as_bool());
            for id in [1, 0x8001, 2] {
                let child = GetDlgItem(Some(bar.0), id).unwrap();
                assert!(IsWindowVisible(child).as_bool());
                assert!(
                    !GetUpdateRect(child, None, false).as_bool(),
                    "unpainted toolbar child {id}"
                );
            }
            let _ = ShowWindow(bar.0, SW_HIDE);
        }
    }
}
