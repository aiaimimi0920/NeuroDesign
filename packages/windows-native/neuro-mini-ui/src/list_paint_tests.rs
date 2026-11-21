//! 字体列表的系统擦底及首帧回归；只检查自有 HWND，不读取用户桌面。
use super::*;

#[test]
fn button_erase_uses_list_background_instead_of_system_white() {
    unsafe {
        let owner = tests::Host::new();
        let popup = Popup::new(owner.0, tests::anchor(), tests::names(40), None).unwrap();
        let dc = GetDC(Some(popup.hwnd));
        let viewport = popup.data.viewport.get();
        let row = popup.data.rows.borrow()[0];
        SetBkColor(dc, COLORREF(0xffffff));
        let brush = SendMessageW(
            viewport,
            WM_CTLCOLORBTN,
            Some(WPARAM(dc.0 as usize)),
            Some(LPARAM(row.0 as isize)),
        );
        let rgb = crate::paint::mix(0x0e1218, popup.data.painter.palette.text, 0.04);
        let expected = COLORREF(((rgb & 255) << 16) | (rgb & 0xff00) | (rgb >> 16));
        let actual = GetBkColor(dc);
        ReleaseDC(Some(popup.hwnd), dc);
        assert_ne!(brush.0, 0, "BUTTON must receive a themed erasure brush");
        assert_eq!(
            actual, expected,
            "system erasure must not flash a light background"
        );
    }
}

#[test]
fn showing_list_synchronously_presents_the_whole_child_tree() {
    unsafe {
        let owner = tests::Host::new();
        let popup = Popup::new(owner.0, tests::anchor(), tests::names(40), Some(0)).unwrap();
        let _ = ShowWindow(popup.hwnd, SW_SHOWNOACTIVATE);
        assert!(
            !GetUpdateRect(popup.hwnd, None, false).as_bool(),
            "first visible root frame is ready"
        );
        assert!(
            !GetUpdateRect(popup.data.viewport.get(), None, false).as_bool(),
            "viewport is in the same frame"
        );
        for row in popup.data.rows.borrow().iter() {
            assert!(
                !GetUpdateRect(*row, None, false).as_bool(),
                "visible rows cannot wait for a later white frame"
            );
        }
        assert!(popup.data.error.borrow().is_none());
    }
}
