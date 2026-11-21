//! 字段切换不发布未初始化的可见控件；保留共用 HWND，返回前完成主题绘制。
use super::*;
use std::cell::RefCell;
use windows::Win32::UI::Shell::*;
thread_local! {
    static CREATED: RefCell<Option<(HWND, Vec<bool>)>> = const { RefCell::new(None) };
}
unsafe extern "system" fn created(code: i32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    if code == HCBT_CREATEWND as i32 {
        let data = &*(lp.0 as *const CBT_CREATEWNDW);
        CREATED.with(|state| {
            if let Some((parent, visible)) = state.borrow_mut().as_mut() {
                if (*data.lpcs).hwndParent == *parent {
                    visible.push((*data.lpcs).style as u32 & WS_VISIBLE.0 != 0);
                }
            }
        });
    }
    CallNextHookEx(None, code, wp, lp)
}
unsafe extern "system" fn count_paints(
    hwnd: HWND,
    msg: u32,
    wp: WPARAM,
    lp: LPARAM,
    _: usize,
    count: usize,
) -> LRESULT {
    if msg == WM_PAINT {
        *(count as *mut usize) += 1;
    }
    DefSubclassProc(hwnd, msg, wp, lp)
}
#[test]
fn switching_schema_stages_new_children_reuses_fields_and_never_uses_white_button_erase() {
    unsafe {
        let (owner, bar) = crate::tests::fixture();
        crate::prepare(bar.0).unwrap();
        let field = |id, kind| PropertyField {
            id,
            label: "属性",
            icon: None,
            kind,
        };
        let base = || {
            vec![
                field(
                    1,
                    PropertyKind::Color {
                        rgb: Some(0x00ff00),
                        stroke: true,
                    },
                ),
                field(
                    2,
                    PropertyKind::Number {
                        value: 3,
                        min: 0,
                        max: 999,
                        suffix: "",
                    },
                ),
            ]
        };
        set_properties(bar.0, 1, base(), true).unwrap();
        let popup = property_window(bar.0).unwrap();
        let _ = ShowWindow(owner.0, SW_SHOWNOACTIVATE);
        let _ = ShowWindow(bar.0, SW_SHOWNOACTIVATE);
        let colour = GetDlgItem(Some(popup), 4).unwrap();
        let value = GetDlgItem(Some(popup), 8).unwrap();
        CREATED.with(|s| *s.borrow_mut() = Some((popup, Vec::new())));
        let hook = SetWindowsHookExW(
            WH_CBT,
            Some(created),
            None,
            GetWindowThreadProcessId(popup, None),
        )
        .unwrap();
        let mut next = base();
        next.push(field(
            3,
            PropertyKind::Number {
                value: 0,
                min: 0,
                max: 999,
                suffix: "",
            },
        ));
        set_properties(bar.0, 2, next, true).unwrap();
        UnhookWindowsHookEx(hook).unwrap();
        let visibility = CREATED.with(|s| s.borrow_mut().take().unwrap().1);
        assert_eq!(
            visibility,
            vec![false; 3],
            "EDIT and steppers must be created hidden"
        );
        assert_eq!(GetDlgItem(Some(popup), 4).unwrap(), colour);
        assert_eq!(GetDlgItem(Some(popup), 8).unwrap(), value);
        assert!(!GetUpdateRect(popup, None, false).as_bool());
        for id in [4, 8, 9, 10, 12, 13, 14] {
            let h = GetDlgItem(Some(popup), id).unwrap();
            assert!(IsWindowVisible(h).as_bool());
            assert!(
                !GetUpdateRect(h, None, false).as_bool(),
                "unfinished child paint {id}"
            );
        }
        let dc = GetDC(Some(colour));
        let brush = SendMessageW(
            popup,
            WM_CTLCOLORBTN,
            Some(WPARAM(dc.0 as usize)),
            Some(LPARAM(colour.0 as isize)),
        );
        let mut paint = LOGBRUSH::default();
        assert_ne!(
            GetObjectW(
                HGDIOBJ(brush.0 as *mut _),
                std::mem::size_of::<LOGBRUSH>() as i32,
                Some((&mut paint as *mut LOGBRUSH).cast())
            ),
            0
        );
        assert_ne!(paint.lbColor, COLORREF(0xffffff));
        assert!(
            paint.lbColor.0 & 255 < 64
                && (paint.lbColor.0 >> 8) & 255 < 64
                && (paint.lbColor.0 >> 16) & 255 < 64
        );
        ReleaseDC(Some(colour), dc);
        set_properties(bar.0, 3, base(), true).unwrap();
        assert_eq!(GetDlgItem(Some(popup), 8).unwrap(), value);
        let mut paints = 0usize;
        SetWindowSubclass(
            popup,
            Some(count_paints),
            1,
            &mut paints as *mut usize as usize,
        )
        .ok()
        .unwrap();
        for _ in 0..8 {
            set_properties(bar.0, 3, base(), true).unwrap();
        }
        RemoveWindowSubclass(popup, Some(count_paints), 1)
            .ok()
            .unwrap();
        assert_eq!(
            paints, 0,
            "unchanged refresh must not synchronously repaint the property bar"
        );
    }
}

struct Projection {
    root: HWND,
    visible_during_layout: Vec<bool>,
}
unsafe extern "system" fn observe_projection(
    hwnd: HWND,
    msg: u32,
    wp: WPARAM,
    lp: LPARAM,
    _: usize,
    data: usize,
) -> LRESULT {
    if msg == WM_WINDOWPOSCHANGED {
        let projection = &mut *(data as *mut Projection);
        projection
            .visible_during_layout
            .push(IsWindowVisible(projection.root).as_bool());
    }
    DefSubclassProc(hwnd, msg, wp, lp)
}
#[test]
fn switching_fields_keeps_old_subtree_until_final_geometry() {
    unsafe {
        let (owner, toolbar) = crate::tests::fixture();
        crate::prepare(toolbar.0).unwrap();
        let number = |id| PropertyField {
            id,
            label: "数值",
            icon: None,
            kind: PropertyKind::Number {
                value: 3,
                min: 0,
                max: 999,
                suffix: "",
            },
        };
        set_properties(toolbar.0, 1, vec![number(1), number(2)], true).unwrap();
        let popup = property_window(toolbar.0).unwrap();
        let _ = ShowWindow(owner.0, SW_SHOWNOACTIVATE);
        let _ = ShowWindow(toolbar.0, SW_SHOWNOACTIVATE);
        assert!(IsWindowVisible(popup).as_bool());
        let input = GetDlgItem(Some(popup), 4).unwrap();
        let mut projection = Projection {
            root: popup,
            visible_during_layout: Vec::new(),
        };
        SetWindowSubclass(
            input,
            Some(observe_projection),
            601,
            &mut projection as *mut Projection as usize,
        )
        .ok()
        .unwrap();
        set_properties(toolbar.0, 2, vec![number(3), number(1), number(2)], true).unwrap();
        RemoveWindowSubclass(input, Some(observe_projection), 601)
            .ok()
            .unwrap();
        assert!(!projection.visible_during_layout.is_empty());
        assert!(
            projection
                .visible_during_layout
                .iter()
                .all(|visible| !visible),
            "partly updated controls were eligible for display: {:?}",
            projection.visible_during_layout
        );
        assert!(
            IsWindowVisible(popup).as_bool(),
            "redraw must be restored after the transaction"
        );
        SetWindowDisplayAffinity(popup, WDA_EXCLUDEFROMCAPTURE).unwrap();
        SetWindowDisplayAffinity(popup, WDA_NONE).unwrap();
        assert!(!GetUpdateRect(popup, None, false).as_bool());
        assert_eq!(input, GetDlgItem(Some(popup), 4).unwrap());
    }
}
