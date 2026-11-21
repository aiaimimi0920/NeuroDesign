//! 只使用自有 HWND 与私有输入队列；不发送全局输入、不枚举产品数据。
use super::*;
pub(super) struct Host(pub(super) HWND);
impl Host {
    pub(super) unsafe fn new() -> Self {
        let hwnd = CreateWindowExW(
            WS_EX_TOOLWINDOW,
            w!("STATIC"),
            w!("List test"),
            WS_POPUP,
            400,
            400,
            400,
            400,
            None,
            None,
            None,
            None,
        )
        .unwrap();
        let _ = ShowWindow(hwnd, SW_SHOWNOACTIVATE);
        Self(hwnd)
    }
}
impl Drop for Host {
    fn drop(&mut self) {
        unsafe {
            let _ = DestroyWindow(self.0);
        }
    }
}
pub(super) fn names(count: usize) -> Vec<String> {
    (0..count).map(|i| format!("Font {i}")).collect()
}
pub(super) fn anchor() -> RECT {
    RECT {
        left: 400,
        top: 400,
        right: 510,
        bottom: 428,
    }
}
#[test]
fn disabled_options_are_real_disabled_buttons_and_cannot_be_chosen_by_click_or_keyboard() {
    unsafe {
        let host = Host::new();
        let mut popup = Popup::new_bounded(
            host.0,
            anchor(),
            vec!["禁用".into(), "可用".into(), "禁用二".into()],
            None,
            true,
        )
        .unwrap();
        popup.data.enabled = vec![false, true, false];
        let d = &popup.data;
        scroll::project(d).unwrap();
        let row = d.rows.borrow()[0];
        assert!(!IsWindowEnabled(row).as_bool());
        SendMessageW(row, BM_CLICK, None, None);
        d.choose(0);
        assert_eq!(d.result.get(), None);
        scroll::tab(d, false).unwrap();
        assert_eq!(d.focus.get(), Some(1));
        d.choose(1);
        assert_eq!(d.result.get(), Some(1));
        assert!(d.closed.get());
    }
}
#[test]
fn large_snapshot_has_a_fixed_visible_button_pool_and_pixel_scrolls_to_last_option() {
    unsafe {
        let host = Host::new();
        let popup = Popup::new(host.0, anchor(), names(10_000), Some(200)).unwrap();
        let d = &popup.data;
        assert!(d.rows.borrow().len() <= 10);
        assert_eq!(d.top.get(), 0.0);
        assert_eq!(d.focus.get(), None);
        scroll::set(d, 17.0).unwrap();
        let row = d.rows.borrow()[0];
        let mut rect = RECT::default();
        GetWindowRect(row, &mut rect).unwrap();
        let mut point = POINT {
            x: rect.left,
            y: rect.top,
        };
        assert!(ScreenToClient(d.viewport.get(), &mut point).as_bool());
        assert_eq!(point.y, d.frame.pad - 17);
        scroll::set(d, d.frame.maximum()).unwrap();
        for hwnd in d.rows.borrow().iter().copied() {
            if GetWindowLongPtrW(hwnd, GWL_STYLE) as u32 & WS_VISIBLE.0 == 0 {
                assert_eq!(GetWindowLongPtrW(hwnd, GWLP_USERDATA), 0);
                SendMessageW(hwnd, BM_CLICK, None, None);
                assert_eq!(d.result.get(), None);
            }
        }
        let row = *d
            .rows
            .borrow()
            .iter()
            .find(|hwnd| GetWindowLongPtrW(**hwnd, GWLP_USERDATA) == 10_000)
            .unwrap();
        let mut label = [0u16; 64];
        let len = GetWindowTextW(row, &mut label);
        assert_eq!(
            String::from_utf16_lossy(&label[..len as usize]),
            "Font 9999"
        );
        SendMessageW(row, BM_CLICK, None, None);
        assert_eq!(d.result.get(), Some(9999));
        assert_eq!(d.selected, Some(200));
    }
}
#[test]
fn wheel_fractional_delta_is_retained_and_scroll_never_selects_an_option() {
    unsafe {
        let host = Host::new();
        let popup = Popup::new(host.0, anchor(), names(300), Some(2)).unwrap();
        for _ in 0..120 {
            scroll::wheel(&popup.data, -1).unwrap();
        }
        assert!((popup.data.top.get() - popup.data.frame.row as f64).abs() < 0.001);
        assert_eq!(popup.data.result.get(), None);
        assert_eq!(popup.data.selected, Some(2));
    }
}
#[test]
fn scrollbar_jump_drag_and_keyboard_scroll_use_the_full_list_without_changing_selection() {
    unsafe {
        let host = Host::new();
        let popup = Popup::new(host.0, anchor(), names(300), Some(2)).unwrap();
        let d = &popup.data;
        let r = d.frame.track();
        scroll::down(
            d,
            POINT {
                x: r.left + 1,
                y: r.bottom - 1,
            },
        )
        .unwrap();
        assert!(d.top.get() > 0.0);
        assert_eq!(d.focus.get(), Some(300));
        scroll::key(d, VK_HOME.0).unwrap();
        let thumb = d.frame.thumb(0.0);
        scroll::down(
            d,
            POINT {
                x: thumb.left + 1,
                y: thumb.top + 1,
            },
        )
        .unwrap();
        assert!(d.drag.get().is_some());
        scroll::motion(
            d,
            POINT {
                x: thumb.left + 1,
                y: r.bottom + 1000,
            },
        )
        .unwrap();
        assert_eq!(d.top.get(), d.frame.maximum());
        scroll::key(d, VK_HOME.0).unwrap();
        assert_eq!(d.top.get(), 0.0);
        scroll::key(d, VK_NEXT.0).unwrap();
        assert_eq!(d.top.get(), d.frame.height() as f64);
        assert_eq!(d.selected, Some(2));
        assert_eq!(d.result.get(), None);
    }
}
#[test]
fn tab_reveals_rows_while_arrow_keys_do_not_select_or_search_fonts() {
    unsafe {
        let host = Host::new();
        let popup = Popup::new(host.0, anchor(), names(30), None).unwrap();
        let d = &popup.data;
        for _ in 0..20 {
            scroll::tab(d, false).unwrap();
        }
        assert_eq!(d.focus.get(), Some(19));
        assert!(d.top.get() > 0.0);
        scroll::key(d, VK_DOWN.0).unwrap();
        assert_eq!(d.focus.get(), Some(19));
        assert_eq!(d.result.get(), None);
        for _ in 0..11 {
            scroll::tab(d, false).unwrap();
        }
        assert_eq!(d.focus.get(), Some(30));
        scroll::key(d, VK_END.0).unwrap();
        assert_eq!(d.top.get(), d.frame.maximum());
    }
}
#[test]
fn layouts_preserve_dip_sizes_and_fit_negative_work_areas_with_upper_fallback() {
    for dpi in [96, 144, 192] {
        let work = RECT {
            left: -1920,
            top: -300,
            right: 0,
            bottom: 740,
        };
        let anchor = RECT {
            left: -20,
            top: 700,
            right: 0,
            bottom: 728,
        };
        let f = geometry::Layout::new(anchor, work, dpi, 1000).unwrap();
        assert_eq!(f.row, crate::layout::px(28, dpi));
        assert_eq!(f.rect.right - f.rect.left, crate::layout::px(196, dpi));
        assert!(f.rect.top >= work.top && f.rect.bottom < anchor.top && f.rect.right <= work.right);
    }
    let f = geometry::Layout::new(
        anchor(),
        RECT {
            left: 0,
            top: 0,
            right: 200,
            bottom: 120,
        },
        96,
        500,
    )
    .unwrap();
    assert_eq!(f.height(), 110);
    assert!(f.maximum() > 0.0);
}
#[test]
fn modal_escape_closes_window_children_capture_and_timer_without_committing() {
    unsafe {
        let host = Host::new();
        let calls = Cell::new(0);
        let handle = Cell::new(HWND::default());
        let result = input::track(host.0, anchor(), names(500), Some(200), || {
            calls.set(calls.get() + 1);
            if calls.get() == 2 {
                let hwnd = FindWindowW(w!("NeuroMini.Native.ScrollingList"), None).unwrap();
                handle.set(hwnd);
                PostMessageW(
                    Some(hwnd),
                    WM_KEYDOWN,
                    WPARAM(VK_ESCAPE.0 as usize),
                    LPARAM(0),
                )
                .unwrap();
            }
            host.0
        })
        .unwrap();
        assert_eq!(result, None);
        assert!(!IsWindow(Some(handle.get())).as_bool());
        assert!(GetCapture().is_invalid());
    }
}
#[test]
fn invalid_snapshots_fail_before_any_list_window_is_created() {
    unsafe {
        let host = Host::new();
        assert!(Popup::new(host.0, anchor(), vec![], None).is_err());
        assert!(Popup::new(host.0, anchor(), names(2), Some(2)).is_err());
        assert!(Popup::new(host.0, anchor(), vec!["bad\0label".into()], None).is_err());
        assert!(Popup::new(host.0, anchor(), names(16_385), None).is_err());
        assert!(input::track(host.0, anchor(), names(2), None, HWND::default).is_err());
    }
}
#[test]
fn real_modal_pointer_messages_commit_a_scrolled_option_beyond_compact_menu_limit() {
    unsafe {
        let host = Host::new();
        let calls = Cell::new(0);
        let result = input::track(host.0, anchor(), names(500), Some(2), || {
            calls.set(calls.get() + 1);
            if calls.get() == 2 {
                let hwnd = FindWindowW(w!("NeuroMini.Native.ScrollingList"), None).unwrap();
                let d = &*(GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const Data);
                scroll::set(d, d.frame.maximum()).unwrap();
                let row = *d
                    .rows
                    .borrow()
                    .iter()
                    .find(|row| GetWindowLongPtrW(**row, GWLP_USERDATA) == 500)
                    .unwrap();
                let mut r = RECT::default();
                GetWindowRect(row, &mut r).unwrap();
                let mut p = POINT {
                    x: r.left + 10,
                    y: (r.top + r.bottom) / 2,
                };
                assert!(ScreenToClient(hwnd, &mut p).as_bool());
                let lp = LPARAM(p.x as u16 as isize | ((p.y as u16 as isize) << 16));
                PostMessageW(Some(hwnd), WM_LBUTTONDOWN, WPARAM(0), lp).unwrap();
                PostMessageW(Some(hwnd), WM_LBUTTONUP, WPARAM(0), lp).unwrap();
            }
            host.0
        })
        .unwrap();
        assert_eq!(result, Some(499));
        assert!(GetCapture().is_invalid());
    }
}
