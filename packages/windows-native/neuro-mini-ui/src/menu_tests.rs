//! 真实菜单 HWND 的有界快照、DPI 行高、禁用项及销毁；不使用系统鼠标输入。
use super::*;
fn entries() -> Vec<MenuItem> {
    vec![
        MenuItem {
            id: 1,
            label: "选择".into(),
            shortcut: "Q".into(),
            icon: Icon::stroke("M5 3v17l5-6h9Z"),
            selected: true,
            enabled: true,
        },
        MenuItem {
            id: 2,
            label: "移动".into(),
            shortcut: "W".into(),
            icon: Icon::stroke("M3 12h18M12 3v18"),
            selected: false,
            enabled: false,
        },
    ]
}
#[test]
fn compact_menu_is_owned_named_and_releases_real_controls() {
    unsafe {
        let (owner, _bar) = crate::tests::fixture();
        let popup = Popup::new(
            owner.0,
            RECT {
                left: 200,
                top: 300,
                right: 228,
                bottom: 328,
            },
            entries(),
            true,
        )
        .unwrap();
        assert_eq!(GetWindow(popup.hwnd, GW_OWNER).unwrap(), owner.0);
        let first = GetDlgItem(Some(popup.hwnd), 1).unwrap();
        let second = GetDlgItem(Some(popup.hwnd), 2).unwrap();
        let mut rect = RECT::default();
        GetWindowRect(first, &mut rect).unwrap();
        assert_eq!(
            rect.bottom - rect.top,
            crate::layout::px(20, popup.data.dpi)
        );
        assert!(IsWindowEnabled(first).as_bool());
        assert!(!IsWindowEnabled(second).as_bool());
        SendMessageW(
            popup.hwnd,
            WM_COMMAND,
            Some(WPARAM(2)),
            Some(LPARAM(second.0 as isize)),
        );
        assert_eq!(popup.data.command.get(), None);
        SendMessageW(
            popup.hwnd,
            WM_COMMAND,
            Some(WPARAM(1)),
            Some(LPARAM(first.0 as isize)),
        );
        assert_eq!(popup.data.command.get(), Some(1));
        let handle = popup.hwnd;
        drop(popup);
        assert!(!IsWindow(Some(handle)).as_bool());
        assert!(!IsWindow(Some(first)).as_bool());
    }
}
#[test]
fn malformed_or_unbounded_snapshot_is_rejected_before_window_creation() {
    unsafe {
        let (owner, _bar) = crate::tests::fixture();
        let anchor = RECT {
            left: 100,
            top: 100,
            right: 128,
            bottom: 128,
        };
        for items in [
            vec![],
            vec![entries()[0].clone(); 13],
            vec![entries()[0].clone(); 2],
        ] {
            assert!(Popup::new(owner.0, anchor, items, false).is_err());
        }
        assert!(track_menu(owner.0, anchor, entries(), false).is_err());
    }
}
#[test]
fn full_menu_loop_selects_without_activating_popup_or_losing_capture() {
    unsafe {
        let (owner, bar) = crate::tests::fixture();
        crate::prepare(bar.0).unwrap();
        let _ = ShowWindow(owner.0, SW_SHOWNOACTIVATE);
        let _ = ShowWindow(bar.0, SW_SHOWNOACTIVATE);
        let body = GetDlgItem(Some(bar.0), 1).unwrap();
        let _ = SetFocus(Some(body));
        let active = GetActiveWindow();
        let mut anchor = RECT::default();
        GetWindowRect(body, &mut anchor).unwrap();
        PostMessageW(
            Some(body),
            WM_KEYDOWN,
            WPARAM(VK_RETURN.0 as usize),
            LPARAM(0),
        )
        .unwrap();
        // 非输入 desktop 无全局前台；只替换该查询为真实线程活动 HWND，消息循环本身完整运行。
        let result =
            input::track_with(bar.0, anchor, entries(), true, || GetActiveWindow()).unwrap();
        assert_eq!(result, Some(1));
        assert_eq!(GetActiveWindow(), active);
        assert_eq!(GetFocus(), body);
        assert!(GetCapture().is_invalid());
    }
}
