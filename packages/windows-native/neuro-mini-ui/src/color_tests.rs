//! 私有 HWND 回归只注入自有窗口消息；不发送全局输入、不采样桌面、不写真实剪贴板。
use super::*;
struct Host(HWND);
impl Host {
    unsafe fn new() -> Self {
        let hwnd = CreateWindowExW(
            WS_EX_TOOLWINDOW,
            w!("STATIC"),
            w!("Color picker test"),
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
fn options() -> ColorOptions {
    ColorOptions {
        initial: Rgba::rgb(239, 68, 68),
        palette: vec![
            Rgba {
                r: 0,
                g: 0,
                b: 0,
                a: 0,
            },
            Rgba::rgb(255, 255, 255),
        ],
        screen_pick: true,
        icons: ColorIcons {
            eyedropper: Icon::stroke(""),
            remove: Icon::stroke(""),
            apply: Icon::stroke(""),
        },
    }
}
#[test]
fn actual_hex_percent_and_palette_only_change_local_draft_until_apply() {
    unsafe {
        let host = Host::new();
        let popup = Popup::new(host.0, geometry::rect(400, 400, 28, 28), options()).unwrap();
        let d = &popup.data;
        controls::changed(popup.hwnd, d, controls::HEX, EN_SETFOCUS as u16);
        let _ = SetWindowTextW(controls::child(popup.hwnd, controls::HEX), w!("#10203080"));
        assert_eq!(
            d.value.get().color(),
            Rgba {
                r: 16,
                g: 32,
                b: 48,
                a: 128
            }
        );
        let _ = SetWindowTextW(controls::child(popup.hwnd, controls::HEX), w!("#1020"));
        assert_eq!(
            controls::text(controls::child(popup.hwnd, controls::HEX)),
            "#1020"
        );
        assert_eq!(d.value.get().color().a, 128);
        assert_eq!(d.action.get(), ColorAction::Cancel);
        let _ = SetWindowTextW(controls::child(popup.hwnd, controls::PERCENT), w!("30"));
        assert_eq!(d.value.get().color().a, 77);
        input::pointer::palette_click(popup.hwnd, d, POINT { x: 39, y: 216 }, false);
        assert_eq!(d.value.get().color().a, 0);
        assert_eq!(d.value.get().color().r, 16);
        SendMessageW(
            controls::child(popup.hwnd, controls::APPLY),
            BM_CLICK,
            None,
            None,
        );
        assert_eq!(d.action.get(), ColorAction::Apply(d.value.get().color()));
        assert!(d.closed.get());
    }
}
#[test]
fn palette_add_remove_survives_cancel_but_does_not_apply_color() {
    unsafe {
        let host = Host::new();
        let popup = Popup::new(host.0, geometry::rect(400, 400, 28, 28), options()).unwrap();
        input::pointer::add(&popup.data);
        input::pointer::add(&popup.data);
        assert_eq!(popup.data.palette.borrow().len(), 3);
        input::pointer::remove(&popup.data, 1);
        assert_eq!(popup.data.removed.borrow().len(), 1);
        SendMessageW(popup.hwnd, WM_CLOSE, None, None);
        assert_eq!(popup.data.action.get(), ColorAction::Cancel);
        assert_eq!(popup.data.palette.borrow().len(), 2);
    }
}
#[test]
fn actual_drag_clamps_values_and_releases_capture_on_cancel_or_destroy() {
    unsafe {
        let host = Host::new();
        let popup = Popup::new(host.0, geometry::rect(400, 400, 28, 28), options()).unwrap();
        assert!(input::pointer::down(
            popup.hwnd,
            &popup.data,
            POINT { x: 10, y: 126 },
            false
        ));
        input::pointer::sample(popup.hwnd, &popup.data, POINT { x: 500, y: -10 });
        assert_eq!(popup.data.value.get().h, 360.0);
        assert_eq!(popup.data.action.get(), ColorAction::Cancel);
        SendMessageW(popup.hwnd, WM_LBUTTONUP, None, None);
        assert!(GetCapture().is_invalid());
        let hwnd = popup.hwnd;
        drop(popup);
        assert!(!IsWindow(Some(hwnd)).as_bool());
    }
}
#[test]
fn modal_escape_returns_palette_changes_and_destroys_all_children() {
    unsafe {
        let host = Host::new();
        let calls = Cell::new(0);
        let popup_handle = Cell::new(HWND::default());
        let result = input::track(host.0, geometry::rect(400, 400, 28, 28), options(), || {
            calls.set(calls.get() + 1);
            if calls.get() == 2 {
                let hwnd = FindWindowW(w!("NeuroMini.Native.ColorPicker"), None).unwrap();
                popup_handle.set(hwnd);
                let d = &*(GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const Data);
                input::pointer::add(d);
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
        assert_eq!(result.action, ColorAction::Cancel);
        assert_eq!(result.palette.len(), 3);
        assert!(!IsWindow(Some(popup_handle.get())).as_bool());
        assert!(GetCapture().is_invalid());
    }
}
#[test]
fn work_area_placement_and_scroll_remain_bounded() {
    let work = geometry::rect(-1920, -300, 1920, 1040);
    for anchor in [
        geometry::rect(-100, 0, 28, 28),
        geometry::rect(-1900, -290, 28, 28),
        geometry::rect(-900, 700, 28, 28),
    ] {
        let r = geometry::anchored(anchor, work, 144);
        assert!(
            r.left >= work.left
                && r.right <= work.right
                && r.top >= work.top
                && r.bottom <= work.bottom
        );
    }
    assert_eq!(geometry::max_scroll(0), 0);
    assert!(geometry::max_scroll(100) > 0);
}
#[test]
fn keyboard_can_add_first_palette_color_and_remove_selected_without_applying() {
    unsafe {
        let host = Host::new();
        let mut config = options();
        config.palette.clear();
        let popup = Popup::new(host.0, geometry::rect(400, 400, 28, 28), config).unwrap();
        input::palette_key(popup.hwnd, &popup.data, VK_SPACE.0);
        assert_eq!(popup.data.palette.borrow().len(), 1);
        input::palette_key(popup.hwnd, &popup.data, VK_RIGHT.0);
        assert_eq!(popup.data.selected.get(), Some(0));
        input::palette_key(popup.hwnd, &popup.data, VK_DELETE.0);
        assert!(popup.data.palette.borrow().is_empty());
        assert_eq!(popup.data.action.get(), ColorAction::Cancel);
    }
}
#[test]
fn small_viewport_scrolls_content_and_reveals_real_focused_edit_without_scaling() {
    unsafe {
        let host = Host::new();
        let popup = Popup::new(host.0, geometry::rect(400, 400, 28, 28), options()).unwrap();
        assert_eq!(
            GetWindowLongPtrW(popup.hwnd, GWL_STYLE) as u32 & (WS_HSCROLL | WS_VSCROLL).0,
            0
        );
        SetWindowPos(
            popup.hwnd,
            None,
            0,
            0,
            crate::layout::px(248, popup.data.dpi),
            crate::layout::px(190, popup.data.dpi),
            SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE,
        )
        .unwrap();
        scroll::configure(popup.hwnd, &popup.data).unwrap();
        scroll::wheel(popup.hwnd, &popup.data, -120);
        assert!(popup.data.origin.get().y > 0);
        let apply = controls::child(popup.hwnd, controls::APPLY);
        let _ = ShowWindow(popup.hwnd, SW_SHOWNOACTIVATE);
        let _ = SetFocus(Some(apply));
        scroll::reveal(popup.hwnd, &popup.data);
        let mut rect = RECT::default();
        let mut client = RECT::default();
        GetWindowRect(apply, &mut rect).unwrap();
        GetClientRect(popup.hwnd, &mut client).unwrap();
        let mut p = POINT {
            x: rect.left,
            y: rect.top,
        };
        assert!(ScreenToClient(popup.hwnd, &mut p).as_bool());
        assert!(p.y >= 0 && p.y + rect.bottom - rect.top <= client.bottom);
        assert_eq!(
            rect.bottom - rect.top,
            crate::layout::px(26, popup.data.dpi)
        );
    }
}
