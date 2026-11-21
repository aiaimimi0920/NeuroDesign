//! 真实面板 viewport、滚轮、焦点、轨道和纯数字单位契约；不采集桌面或全局输入。
use super::*;
fn fields() -> Vec<PropertyField> {
    (1..=16)
        .map(|id| PropertyField {
            id,
            label: "百分比",
            icon: None,
            kind: PropertyKind::Number {
                value: 50,
                min: 0,
                max: 100,
                suffix: "%",
            },
        })
        .collect()
}
unsafe fn input(panel: &PropertyPanel, index: usize) -> HWND {
    panel.0.data.controls.borrow()[index].hwnd
}
unsafe fn visible(panel: &PropertyPanel) {
    let d = &panel.0.data;
    panel
        .place(RECT {
            left: 40,
            top: 40,
            right: 40 + d.width.get(),
            bottom: 40 + d.height.get(),
        })
        .unwrap();
    panel.show(true);
}
unsafe fn text(hwnd: HWND) -> String {
    let mut text = [0; 32];
    let n = GetWindowTextW(hwnd, &mut text) as usize;
    String::from_utf16_lossy(&text[..n])
}
#[test]
fn height_constraint_keeps_fixed_rows_and_handles_and_restores_full_height() {
    unsafe {
        let (_owner, bar) = crate::tests::fixture();
        crate::prepare(bar.0).unwrap();
        let panel = PropertyPanel::new(bar.0, 125).unwrap();
        panel.update(1, fields(), true).unwrap();
        let d = &panel.0.data;
        let px = |v| crate::layout::px(v, d.dpi.get());
        let original = panel.size();
        let child = input(&panel, 15);
        panel.set_max_height(px(100)).unwrap();
        visible(&panel);
        assert_eq!(panel.size(), (px(125), px(100)));
        assert!(scroll::maximum(d) > 0.0);
        assert_eq!(GetParent(child).unwrap(), d.viewport.get());
        assert_eq!(GetWindowLongW(child, GWL_STYLE) & ES_RIGHT, ES_RIGHT);
        let frame = d.controls.borrow()[15].frame;
        assert_eq!(frame.bottom - frame.top, px(28));
        assert_eq!(frame.right - frame.left, px(109));
        panel.set_max_height(px(800)).unwrap();
        assert_eq!(panel.size(), original);
        assert_eq!(input(&panel, 15), child);
        assert_eq!(scroll::maximum(d), 0.0);
        assert!(panel.set_max_height(px(35)).is_err());
        let viewport = d.viewport.get();
        drop(panel);
        assert!(!IsWindow(Some(viewport)).as_bool());
        assert!(!IsWindow(Some(child)).as_bool());
    }
}
#[test]
fn wheel_over_edit_scrolls_pixels_not_values_and_preserves_same_context_draft() {
    unsafe {
        let (_owner, bar) = crate::tests::fixture();
        crate::prepare(bar.0).unwrap();
        let panel = PropertyPanel::new(bar.0, 125).unwrap();
        panel.update(1, fields(), true).unwrap();
        let d = &panel.0.data;
        let row = crate::layout::px(30, d.dpi.get());
        panel.set_max_height(row * 3).unwrap();
        visible(&panel);
        let child = input(&panel, 0);
        SetWindowTextW(child, w!("77")).unwrap();
        SendMessageW(
            child,
            WM_MOUSEWHEEL,
            Some(WPARAM(((-60i16) as u16 as usize) << 16)),
            None,
        );
        assert_eq!(d.scroll_top.get(), row as f64 / 2.0);
        assert_eq!(text(child), "77");
        assert!(panel.take_edit(1).is_none());
        let top = d.scroll_top.get();
        panel.update(1, fields(), true).unwrap();
        assert_eq!(d.scroll_top.get(), top);
        assert_eq!(text(child), "77");
        SendMessageW(child, WM_KEYDOWN, Some(WPARAM(VK_RETURN.0 as usize)), None);
        assert_eq!(panel.take_edit(1).unwrap().value.as_deref(), Some("77"));
        panel.update(2, fields(), true).unwrap();
        assert_eq!(d.scroll_top.get(), top);
        assert_eq!(text(child), "50");
    }
}
#[test]
fn tab_and_programmatic_focus_reveal_whole_field_and_number_arrows_still_step() {
    unsafe {
        let (owner, bar) = crate::tests::fixture();
        crate::prepare(bar.0).unwrap();
        let _ = ShowWindow(owner.0, SW_SHOWNOACTIVATE);
        let _ = ShowWindow(bar.0, SW_SHOWNOACTIVATE);
        let panel = PropertyPanel::new(bar.0, 125).unwrap();
        panel.update(1, fields(), true).unwrap();
        let d = &panel.0.data;
        panel
            .set_max_height(crate::layout::px(100, d.dpi.get()))
            .unwrap();
        visible(&panel);
        let last = input(&panel, 15);
        let _ = SetFocus(Some(last));
        assert_eq!(GetFocus(), last);
        assert_eq!(d.scroll_top.get(), scroll::maximum(d));
        SendMessageW(last, WM_KEYDOWN, Some(WPARAM(VK_UP.0 as usize)), None);
        assert_eq!(panel.take_edit(16).unwrap().value.as_deref(), Some("51"));
        assert_eq!(d.scroll_top.get(), scroll::maximum(d));
        let first = input(&panel, 0);
        let _ = SetFocus(Some(first));
        assert_eq!(d.scroll_top.get(), 0.0);
        let second = input(&panel, 1);
        assert!(panel.translate(&MSG {
            hwnd: first,
            message: WM_KEYDOWN,
            wParam: WPARAM(VK_TAB.0 as usize),
            ..Default::default()
        }));
        assert_eq!(GetFocus(), second);
    }
}
#[test]
fn track_drag_and_root_keys_scroll_without_committing_and_capture_is_reclaimed() {
    unsafe {
        let (owner, bar) = crate::tests::fixture();
        crate::prepare(bar.0).unwrap();
        let _ = ShowWindow(owner.0, SW_SHOWNOACTIVATE);
        let _ = ShowWindow(bar.0, SW_SHOWNOACTIVATE);
        let panel = PropertyPanel::new(bar.0, 125).unwrap();
        panel.update(1, fields(), true).unwrap();
        let d = &panel.0.data;
        panel
            .set_max_height(crate::layout::px(100, d.dpi.get()))
            .unwrap();
        visible(&panel);
        let hwnd = panel.window().unwrap();
        let t = scroll::thumb(d);
        let lp = |x: i32, y: i32| LPARAM(((x as u16 as u32) | ((y as u16 as u32) << 16)) as isize);
        SendMessageW(hwnd, WM_LBUTTONDOWN, None, Some(lp(t.left + 1, t.top + 1)));
        assert_eq!(GetCapture(), hwnd);
        SendMessageW(hwnd, WM_MOUSEMOVE, None, Some(lp(t.left + 1, t.top + 30)));
        assert!(d.scroll_top.get() > 0.0);
        SendMessageW(hwnd, WM_LBUTTONUP, None, None);
        assert_ne!(GetCapture(), hwnd);
        SendMessageW(hwnd, WM_KEYDOWN, Some(WPARAM(VK_END.0 as usize)), None);
        assert_eq!(d.scroll_top.get(), scroll::maximum(d));
        SendMessageW(hwnd, WM_KEYDOWN, Some(WPARAM(VK_HOME.0 as usize)), None);
        assert_eq!(d.scroll_top.get(), 0.0);
        for id in 1..=16 {
            assert!(panel.take_edit(id).is_none());
        }
        SendMessageW(hwnd, WM_LBUTTONDOWN, None, Some(lp(t.left + 1, t.top + 1)));
        panel.show(false);
        assert_ne!(
            GetCapture(),
            hwnd,
            "hiding a panel must release its scrollbar capture"
        );
        visible(&panel);
        SendMessageW(hwnd, WM_LBUTTONDOWN, None, Some(lp(t.left + 1, t.top + 1)));
        drop(panel);
        assert_ne!(GetCapture(), hwnd);
    }
}
#[test]
fn number_suffix_is_outside_real_edit_and_never_enters_value_submission() {
    unsafe {
        let (_owner, bar) = crate::tests::fixture();
        crate::prepare(bar.0).unwrap();
        let panel = PropertyPanel::new(bar.0, 125).unwrap();
        panel.update(1, fields(), true).unwrap();
        let d = &panel.0.data;
        let child = input(&panel, 0);
        let frame = d.controls.borrow()[0].frame;
        let mut rect = RECT::default();
        GetClientRect(child, &mut rect).unwrap();
        assert!(rect.right <= frame.right - frame.left - crate::layout::px(48, d.dpi.get()));
        assert_eq!(text(child), "50");
        SetWindowTextW(child, w!("100")).unwrap();
        SendMessageW(child, WM_KEYDOWN, Some(WPARAM(VK_RETURN.0 as usize)), None);
        assert_eq!(panel.take_edit(1).unwrap().value.as_deref(), Some("100"));
        let mut invalid = fields();
        if let PropertyKind::Number { suffix, .. } = &mut invalid[0].kind {
            *suffix = "bad\n";
        }
        assert!(panel.update(1, invalid, true).is_err());
    }
}
