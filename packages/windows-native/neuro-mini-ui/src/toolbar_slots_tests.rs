//! 动态文字槽保持 HWND/焦点，隐藏槽零占位；异步通知携带本次投影凭证。
use crate::tests;
use crate::toolbar::{self, slots};
use crate::*;
use windows::Win32::{
    Foundation::*,
    Graphics::Gdi::*,
    UI::{Controls::*, WindowsAndMessaging::*},
};
unsafe fn fixture() -> (tests::Window, tests::Window) {
    let (owner, _) = tests::fixture();
    let bar = tests::Window(
        create(
            owner.0,
            "text slots",
            WM_APP + 11,
            &[
                Item::Button(Button {
                    id: 1,
                    label: "固定",
                    icon: Icon::stroke("M1 1 20 20"),
                    dropdown: true,
                }),
                Item::Slot(2),
                Item::Slot(3),
            ],
        )
        .unwrap(),
    );
    prepare(bar.0).unwrap();
    (owner, bar)
}
fn slot(id: u16, group: bool, token: u32) -> TextSlot<'static> {
    TextSlot {
        id,
        text: "工具",
        label: "扩展工具",
        dropdown: group,
        active: false,
        enabled: true,
        context: token,
    }
}
#[test]
fn hidden_slots_have_no_space_and_updates_preserve_fixed_controls_and_drafts() {
    unsafe {
        let (_owner, bar) = fixture();
        let fixed = GetDlgItem(Some(bar.0), 1).unwrap();
        let dynamic = GetDlgItem(Some(bar.0), 2).unwrap();
        let mut before = RECT::default();
        GetWindowRect(bar.0, &mut before).unwrap();
        assert_eq!(before.right - before.left, 40);
        assert_eq!(GetWindowLongW(dynamic, GWL_STYLE) as u32 & WS_VISIBLE.0, 0);
        set_text_slots(bar.0, &[slot(2, false, 7)]).unwrap();
        assert_eq!(toolbar::ui(bar.0).unwrap().layout.borrow().width, 68);
        assert_eq!(
            GetWindowLongW(GetDlgItem(Some(bar.0), 0x8002).unwrap(), GWL_STYLE) as u32
                & WS_VISIBLE.0,
            0
        );
        set_text_slots(bar.0, &[slot(3, true, 8)]).unwrap();
        assert_eq!(toolbar::ui(bar.0).unwrap().layout.borrow().width, 68);
        assert_eq!(GetWindowLongW(dynamic, GWL_STYLE) as u32 & WS_VISIBLE.0, 0);
        assert_eq!(GetDlgItem(Some(bar.0), 1).unwrap(), fixed);
        assert_eq!(GetDlgItem(Some(bar.0), 2).unwrap(), dynamic);
        set_text_slots(bar.0, &[]).unwrap();
        assert_eq!(toolbar::ui(bar.0).unwrap().layout.borrow().width, 40);
    }
}
#[test]
fn direct_and_group_buttons_post_their_context_and_hidden_or_disabled_slots_do_not() {
    unsafe {
        let (owner, bar) = fixture();
        set_text_slots(bar.0, &[slot(2, false, 7), slot(3, true, 8)]).unwrap();
        for (id, token, kind) in [(2, 7, 0), (3, 8, 0), (0x8003, 8, 1)] {
            SendMessageW(GetDlgItem(Some(bar.0), id).unwrap(), BM_CLICK, None, None);
            let mut m = MSG::default();
            assert!(
                PeekMessageW(&mut m, Some(owner.0), WM_APP + 11, WM_APP + 11, PM_REMOVE).as_bool()
            );
            assert_eq!(
                (m.wParam.0 as u16, m.wParam.0 >> 32, m.lParam.0),
                (id as u16 & 0x7fff, token, kind)
            );
        }
        set_text_slots(bar.0, &[]).unwrap();
        for id in [2, 3, 0x8003] {
            SendMessageW(GetDlgItem(Some(bar.0), id).unwrap(), BM_CLICK, None, None);
        }
        assert!(!PeekMessageW(
            &mut MSG::default(),
            Some(owner.0),
            WM_APP + 11,
            WM_APP + 11,
            PM_REMOVE
        )
        .as_bool());
    }
}
#[test]
fn replacing_a_pressed_slot_cancels_its_gesture_and_rejects_a_queued_old_hold_notification() {
    unsafe {
        let (owner, bar) = fixture();
        let _ = ShowWindow(owner.0, SW_SHOWNOACTIVATE);
        let _ = ShowWindow(bar.0, SW_SHOWNOACTIVATE);
        set_text_slots(bar.0, &[slot(2, true, 7)]).unwrap();
        let button = GetDlgItem(Some(bar.0), 2).unwrap();
        SendMessageW(
            button,
            WM_LBUTTONDOWN,
            Some(WPARAM(1)),
            Some(LPARAM(4 | (4 << 16))),
        );
        set_text_slots(bar.0, &[slot(2, true, 8)]).unwrap();
        assert_ne!(
            windows::Win32::UI::Input::KeyboardAndMouse::GetCapture(),
            button
        );
        SendMessageW(
            bar.0,
            WM_COMMAND,
            Some(WPARAM(0x8002 | (7usize << 32))),
            Some(LPARAM(button.0 as isize)),
        );
        SendMessageW(button, WM_LBUTTONUP, None, Some(LPARAM(4 | (4 << 16))));
        assert!(!PeekMessageW(
            &mut MSG::default(),
            Some(owner.0),
            WM_APP + 11,
            WM_APP + 11,
            PM_REMOVE
        )
        .as_bool());
    }
}
#[test]
fn text_slot_paints_real_signal_text_and_rejects_invalid_updates_without_hiding_existing_content() {
    unsafe {
        let (_owner, bar) = fixture();
        set_text_slots(bar.0, &[slot(2, false, 1)]).unwrap();
        assert!(set_text_slots(bar.0, &[slot(1, false, 1)]).is_err());
        let data = toolbar::ui(bar.0).unwrap();
        let content = data.content.borrow();
        let content = content.as_ref().unwrap();
        let control = content.controls.iter().find(|c| c.spec.id == 2).unwrap();
        assert!(control.visible.get());
        let dc = CreateCompatibleDC(None);
        let info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: 28,
                biHeight: -28,
                biPlanes: 1,
                biBitCount: 32,
                ..Default::default()
            },
            ..Default::default()
        };
        let mut pixels = std::ptr::null_mut();
        let bitmap =
            CreateDIBSection(Some(dc), &info, DIB_RGB_COLORS, &mut pixels, None, 0).unwrap();
        let old = SelectObject(dc, HGDIOBJ(bitmap.0));
        let rect = RECT {
            right: 28,
            bottom: 28,
            ..Default::default()
        };
        let state = crate::paint::ButtonState {
            active: false,
            enabled: true,
            focus: false,
            hot: false,
            pressed: false,
            corner: false,
            dropdown: false,
            size: 28,
        };
        content
            .painter
            .button(dc, rect, Icon::stroke(""), state)
            .unwrap();
        slots::paint(
            content,
            control,
            &DRAWITEMSTRUCT {
                hDC: dc,
                rcItem: rect,
                ..Default::default()
            },
            state,
            96,
        )
        .unwrap();
        GdiFlush().ok().unwrap();
        let bytes = std::slice::from_raw_parts(pixels as *const u8, 28 * 28 * 4);
        assert!(bytes
            .chunks_exact(4)
            .any(|p| p[1] > 140 && p[2] > 120 && p[0] < 100));
        SelectObject(dc, old);
        let _ = DeleteObject(HGDIOBJ(bitmap.0));
        let _ = DeleteDC(dc);
    }
}
