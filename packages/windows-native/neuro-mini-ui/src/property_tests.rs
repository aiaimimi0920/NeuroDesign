//! 真实 EDIT 的草稿、同目标刷新、提交合并与工具条附着，不使用外部窗口或输入注入。
use super::*;
use windows::Win32::UI::Shell::*;
fn fields(value: i32) -> Vec<PropertyField> {
    vec![
        PropertyField {
            id: 1,
            label: "线宽",
            icon: None,
            kind: PropertyKind::Number {
                value,
                min: 0,
                max: 999,
                suffix: "",
            },
        },
        PropertyField {
            id: 2,
            label: "箭头",
            icon: None,
            kind: PropertyKind::Action {
                value: String::new(),
                active: false,
                width: 28,
            },
        },
    ]
}
unsafe fn first(hwnd: HWND) -> HWND {
    let ui = toolbar::ui(hwnd).unwrap();
    let properties = ui.properties.borrow();
    let value = properties.as_ref().unwrap().data.controls.borrow()[0].hwnd;
    value
}
#[test]
fn property_draft_survives_refresh_but_not_target_change_and_edits_are_bounded() {
    unsafe {
        let (_owner, bar) = crate::tests::fixture();
        crate::prepare(bar.0).unwrap();
        set_properties(bar.0, 1, fields(3), true).unwrap();
        let child = first(bar.0);
        SetWindowTextW(child, w!("20")).unwrap();
        set_properties(bar.0, 1, fields(3), true).unwrap();
        let mut text = [0; 16];
        let n = GetWindowTextW(child, &mut text) as usize;
        assert_eq!(String::from_utf16_lossy(&text[..n]), "20");
        SendMessageW(child, WM_KEYDOWN, Some(WPARAM(VK_UP.0 as usize)), None);
        let edit = take_property_edit(bar.0, 1).unwrap();
        assert_eq!(edit.context, 1);
        assert_eq!(edit.value.as_deref(), Some("21"));
        assert!(take_property_edit(bar.0, 1).is_none());
        SetWindowTextW(child, w!("99")).unwrap();
        SendMessageW(child, WM_KEYDOWN, Some(WPARAM(VK_ESCAPE.0 as usize)), None);
        assert!(take_property_edit(bar.0, 1).is_none());
        let n = GetWindowTextW(child, &mut text) as usize;
        assert_eq!(String::from_utf16_lossy(&text[..n]), "21");
        set_properties(bar.0, 2, fields(6), true).unwrap();
        let n = GetWindowTextW(child, &mut text) as usize;
        assert_eq!(String::from_utf16_lossy(&text[..n]), "6");
        assert_eq!(child, first(bar.0));
    }
}
#[test]
fn property_popup_follows_shell_hides_with_it_and_is_destroyed_with_owner() {
    unsafe {
        let (owner, bar) = crate::tests::fixture();
        crate::prepare(bar.0).unwrap();
        set_properties(bar.0, 1, fields(3), true).unwrap();
        let prop = toolbar::ui(bar.0)
            .unwrap()
            .properties
            .borrow()
            .as_ref()
            .unwrap()
            .hwnd;
        let _ = ShowWindow(owner.0, SW_SHOWNOACTIVATE);
        let _ = ShowWindow(bar.0, SW_SHOWNOACTIVATE);
        assert!(IsWindowVisible(prop).as_bool());
        SetWindowPos(
            bar.0,
            None,
            300,
            400,
            0,
            0,
            SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
        )
        .unwrap();
        let mut a = RECT::default();
        let mut b = RECT::default();
        GetWindowRect(bar.0, &mut a).unwrap();
        GetWindowRect(prop, &mut b).unwrap();
        assert_eq!(a.left, b.left);
        assert_eq!(a.top, b.bottom);
        let _ = ShowWindow(bar.0, SW_HIDE);
        assert!(!IsWindowVisible(prop).as_bool());
        drop(bar);
        assert!(!IsWindow(Some(prop)).as_bool());
    }
}
unsafe extern "system" fn refresh_on_prepare(
    hwnd: HWND,
    msg: u32,
    wp: WPARAM,
    lp: LPARAM,
    _: usize,
    toolbar: usize,
) -> LRESULT {
    if msg == WM_APP + 1 && lp.0 == crate::PREPARED {
        set_properties(HWND(toolbar as *mut _), 1, fields(3), true).unwrap();
        return LRESULT(0);
    }
    DefSubclassProc(hwnd, msg, wp, lp)
}
#[test]
fn synchronous_host_refresh_during_show_does_not_hide_prepared_properties() {
    unsafe {
        let (owner, bar) = crate::tests::fixture();
        crate::prepare(bar.0).unwrap();
        set_properties(bar.0, 1, fields(3), true).unwrap();
        SetWindowSubclass(owner.0, Some(refresh_on_prepare), 1, bar.0 .0 as usize)
            .ok()
            .unwrap();
        let _ = ShowWindow(owner.0, SW_SHOWNOACTIVATE);
        let _ = ShowWindow(bar.0, SW_SHOWNOACTIVATE);
        assert!(IsWindowVisible(property_window(bar.0).unwrap()).as_bool());
        RemoveWindowSubclass(owner.0, Some(refresh_on_prepare), 1)
            .ok()
            .unwrap();
    }
}
#[test]
fn dropdown_keeps_icon_value_and_width_and_expands_without_disabling_other_properties() {
    unsafe {
        let (_owner, bar) = crate::tests::fixture();
        crate::prepare(bar.0).unwrap();
        let mut items = fields(3);
        items.push(PropertyField {
            id: 3,
            label: "字体",
            icon: Some(Icon::stroke("M3 3h18M12 3v18")),
            kind: PropertyKind::Dropdown {
                value: "Microsoft YaHei".into(),
                width: 110,
            },
        });
        set_properties(bar.0, 1, items.clone(), true).unwrap();
        let ui = toolbar::ui(bar.0).unwrap();
        let prop = ui.properties.borrow();
        let data = &prop.as_ref().unwrap().data;
        let hwnd = data.controls.borrow()[2].hwnd;
        set_property_expanded(bar.0, 3, true).unwrap();
        assert!(data.controls.borrow()[2].expanded);
        assert!(data
            .controls
            .borrow()
            .iter()
            .all(|c| IsWindowEnabled(c.hwnd).as_bool()));
        let mut rect = RECT::default();
        GetWindowRect(hwnd, &mut rect).unwrap();
        assert_eq!(
            rect.right - rect.left,
            crate::layout::px(110, data.dpi.get())
        );
        drop(prop);
        set_property_expanded(bar.0, 3, false).unwrap();
        set_properties(bar.0, 1, items, true).unwrap();
        assert_eq!(
            ui.properties
                .borrow()
                .as_ref()
                .unwrap()
                .data
                .controls
                .borrow()[2]
                .hwnd,
            hwnd
        );
        assert!(
            !ui.properties
                .borrow()
                .as_ref()
                .unwrap()
                .data
                .controls
                .borrow()[2]
                .expanded
        );
    }
}
