//! 纵向面板复用真实 EDIT 的端到端通知、只读字段、草稿与 HWND 回收。
use super::*;
fn fields() -> Vec<PropertyField> {
    let mut result = Vec::new();
    for id in 1..=4 {
        result.push(PropertyField {
            id,
            label: "操作",
            icon: None,
            kind: PropertyKind::Action {
                value: String::new(),
                active: false,
                width: 28,
            },
        });
    }
    result.push(PropertyField {
        id: 5,
        label: "宽度",
        icon: None,
        kind: PropertyKind::Number {
            value: 320,
            min: 32,
            max: 8192,
            suffix: "",
        },
    });
    result.push(PropertyField {
        id: 6,
        label: "高度",
        icon: None,
        kind: PropertyKind::ReadOnly {
            value: "240".into(),
        },
    });
    result
}
#[test]
fn panel_stacks_numbers_at_full_width_and_keeps_readonly_height_out_of_edit_notifications() {
    unsafe {
        let (_owner, bar) = crate::tests::fixture();
        crate::prepare(bar.0).unwrap();
        let panel = PropertyPanel::new(bar.0, 125).unwrap();
        panel.update(1, fields(), true).unwrap();
        let dpi = panel.0.data.dpi.get();
        assert_eq!(
            panel.size(),
            (crate::layout::px(125, dpi), crate::layout::px(96, dpi))
        );
        let controls = panel.0.data.controls.borrow();
        assert_eq!(controls[0].frame.top, controls[3].frame.top);
        assert!(controls[4].frame.top > controls[3].frame.top);
        assert_eq!(
            controls[4].frame.right - controls[4].frame.left,
            crate::layout::px(117, dpi)
        );
        let read = controls[5].hwnd;
        assert_ne!(GetWindowLongW(read, GWL_STYLE) & ES_READONLY, 0);
        drop(controls);
        SendMessageW(read, WM_KEYDOWN, Some(WPARAM(VK_UP.0 as usize)), None);
        assert!(panel.take_edit(6).is_none());
    }
}
#[test]
fn panel_accepts_four_digit_width_preserves_draft_and_routes_separate_notifications() {
    unsafe {
        let (owner, bar) = crate::tests::fixture();
        crate::prepare(bar.0).unwrap();
        let panel = PropertyPanel::new(bar.0, 125).unwrap();
        panel.update(12, fields(), true).unwrap();
        let input = GetDlgItem(Some(panel.0.data.viewport.get()), 20).unwrap();
        assert_eq!(SendMessageW(input, EM_GETLIMITTEXT, None, None).0, 4);
        SetWindowTextW(input, w!("8192")).unwrap();
        panel.update(12, fields(), true).unwrap();
        let mut text = [0; 8];
        let n = GetWindowTextW(input, &mut text) as usize;
        assert_eq!(String::from_utf16_lossy(&text[..n]), "8192");
        SendMessageW(input, WM_KEYDOWN, Some(WPARAM(VK_RETURN.0 as usize)), None);
        let mut msg = MSG::default();
        let mut found = false;
        for _ in 0..16 {
            if !PeekMessageW(&mut msg, Some(owner.0), WM_APP + 1, WM_APP + 1, PM_REMOVE).as_bool() {
                break;
            }
            if msg.lParam.0 == PANEL && msg.wParam.0 == 5 {
                found = true;
                break;
            }
        }
        assert!(found);
        let edit = panel.take_edit(5).unwrap();
        assert_eq!(edit.context, 12);
        assert_eq!(edit.value.as_deref(), Some("8192"));
        panel.update(13, fields(), true).unwrap();
        let n = GetWindowTextW(input, &mut text) as usize;
        assert_eq!(String::from_utf16_lossy(&text[..n]), "320");
    }
}
#[test]
fn panel_close_does_not_close_toolbar_and_destroying_toolbar_reclaims_panel() {
    unsafe {
        let (_owner, bar) = crate::tests::fixture();
        crate::prepare(bar.0).unwrap();
        let panel = PropertyPanel::new(bar.0, 125).unwrap();
        panel.update(1, fields(), true).unwrap();
        let hwnd = panel.window().unwrap();
        panel.show(true);
        assert!(panel.is_open());
        SendMessageW(hwnd, WM_CLOSE, None, None);
        assert!(!panel.is_open());
        assert!(!IsWindowVisible(hwnd).as_bool());
        assert!(IsWindow(Some(bar.0)).as_bool());
        drop(bar);
        assert!(!IsWindow(Some(hwnd)).as_bool());
        assert!(panel.window().is_none());
    }
}

#[test]
fn mixed_field_availability_is_stable_and_preserves_handles_focus_and_drafts() {
    unsafe {
        let (owner, bar) = crate::tests::fixture();
        crate::prepare(bar.0).unwrap();
        let _ = ShowWindow(owner.0, SW_SHOWNOACTIVATE);
        let _ = ShowWindow(bar.0, SW_SHOWNOACTIVATE);
        let panel = PropertyPanel::new(bar.0, 125).unwrap();
        let states = || {
            fields()
                .into_iter()
                .map(|f| {
                    let enabled = f.id != 1 && f.id != 6;
                    (f, enabled)
                })
                .collect()
        };
        panel.update_with_states(7, states()).unwrap();
        panel.show(true);
        let input = panel.0.data.controls.borrow()[4].hwnd;
        let _ = SetFocus(Some(input));
        SetWindowTextW(input, w!("1234")).unwrap();
        for _ in 0..8 {
            panel.update_with_states(7, states()).unwrap();
            assert!(controls::update(&panel.0, 7, states(), Vec::new())
                .unwrap()
                .is_none());
            assert_eq!(panel.0.data.controls.borrow()[4].hwnd, input);
            assert_eq!(GetFocus(), input);
        }
        for c in panel.0.data.controls.borrow().iter() {
            for h in c.handles() {
                assert_eq!(
                    IsWindowEnabled(h).as_bool(),
                    c.field.id != 1 && c.field.id != 6
                );
            }
        }
        let mut text = [0; 16];
        let n = GetWindowTextW(input, &mut text) as usize;
        assert_eq!(String::from_utf16_lossy(&text[..n]), "1234");
        panel
            .update_with_states(8, fields().into_iter().map(|f| (f, false)).collect())
            .unwrap();
        for c in panel.0.data.controls.borrow().iter() {
            assert!(c.handles().all(|h| !IsWindowEnabled(h).as_bool()));
        }
        let n = GetWindowTextW(input, &mut text) as usize;
        assert_eq!(String::from_utf16_lossy(&text[..n]), "320");
    }
}

#[test]
fn field_anchor_tracks_scroll_and_does_not_survive_field_or_window_removal() {
    unsafe {
        let (owner, bar) = crate::tests::fixture();
        crate::prepare(bar.0).unwrap();
        let _ = ShowWindow(owner.0, SW_SHOWNOACTIVATE);
        let _ = ShowWindow(bar.0, SW_SHOWNOACTIVATE);
        let panel = PropertyPanel::new(bar.0, 125).unwrap();
        panel.update(1, fields(), true).unwrap();
        panel
            .set_max_height(crate::layout::px(60, panel.0.data.dpi.get()))
            .unwrap();
        panel.show(true);
        let before = panel.anchor(6).unwrap();
        let child = panel.0.data.controls.borrow()[5].hwnd;
        let _ = SetFocus(Some(child));
        let after = panel.anchor(6).unwrap();
        assert!(after.top < before.top);
        let mut actual = RECT::default();
        GetWindowRect(child, &mut actual).unwrap();
        assert_eq!(after, actual);
        assert!(panel.anchor(99).is_none());
        panel
            .update(
                2,
                fields().into_iter().filter(|f| f.id != 6).collect(),
                true,
            )
            .unwrap();
        assert!(panel.anchor(6).is_none());
        drop(bar);
        assert!(panel.anchor(5).is_none());
    }
}
