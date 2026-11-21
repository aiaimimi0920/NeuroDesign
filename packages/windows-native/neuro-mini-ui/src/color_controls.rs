//! 真实 EDIT 保留原生文本输入；菜单草稿与最终颜色彼此独立，控件只在值实际改变时刷新。
use super::*;
pub(super) const HEX: i32 = 100;
pub(super) const PERCENT: i32 = 101;
pub(super) const PREVIEW: i32 = 102;
pub(super) const PICK: i32 = 103;
pub(super) const CANCEL: i32 = 104;
pub(super) const APPLY: i32 = 105;
pub(super) const PALETTE: i32 = 106;
pub(super) unsafe fn child(hwnd: HWND, id: i32) -> HWND {
    GetDlgItem(Some(hwnd), id).unwrap_or_default()
}
pub(super) unsafe fn text(hwnd: HWND) -> String {
    let mut text = [0u16; 64];
    let len = GetWindowTextW(hwnd, &mut text);
    String::from_utf16_lossy(&text[..len.max(0) as usize])
}
unsafe fn set(hwnd: HWND, value: &str) {
    if text(hwnd) != value {
        let _ = SetWindowTextW(hwnd, &HSTRING::from(value));
    }
}
pub(super) unsafe fn create(p: &Popup) -> Result<()> {
    let instance = HINSTANCE(GetModuleHandleW(None)?.0);
    let px = |v| crate::layout::px(v, p.data.dpi);
    for (id, label, r, edit) in [
        (
            PREVIEW,
            "复制当前颜色 HEX；右键复制 RGB",
            geometry::rect(9, 173, 26, 26),
            false,
        ),
        (PICK, "屏幕取色", geometry::rect(39, 173, 26, 26), false),
        (HEX, "", geometry::rect(69, 173, 130, 26), true),
        (PERCENT, "", geometry::rect(203, 173, 26, 26), true),
        (PALETTE, "调色板", geometry::PALETTE, false),
        (CANCEL, "取消", geometry::rect(9, 283, 48, 26), false),
        (APPLY, "应用", geometry::rect(169, 283, 70, 26), false),
    ] {
        let style = WS_CHILD
            | WS_VISIBLE
            | WS_TABSTOP
            | WINDOW_STYLE(if edit {
                ES_AUTOHSCROLL as u32
            } else {
                BS_OWNERDRAW as u32
            });
        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            if edit { w!("EDIT") } else { w!("BUTTON") },
            &HSTRING::from(label),
            style,
            px(r.left),
            px(r.top),
            px(r.right - r.left),
            px(r.bottom - r.top),
            Some(p.hwnd),
            Some(HMENU(id as *mut _)),
            Some(instance),
            None,
        )?;
        SendMessageW(hwnd, WM_SETFONT, Some(WPARAM(p.data.font.0 as usize)), None);
        if edit {
            SendMessageW(
                hwnd,
                EM_SETLIMITTEXT,
                Some(WPARAM(if id == HEX { 32 } else { 8 })),
                None,
            );
        }
    }
    let _ = EnableWindow(child(p.hwnd, PICK), p.data.screen_pick);
    refresh(p.hwnd, &p.data);
    Ok(())
}
pub(super) unsafe fn refresh(hwnd: HWND, d: &Data) {
    d.updating.set(true);
    if !d.hex_editing.get() {
        set(child(hwnd, HEX), &d.value.get().color().hex());
    }
    if GetFocus() != child(hwnd, PERCENT) {
        set(
            child(hwnd, PERCENT),
            &format!("{}", (d.value.get().a * 100.0).round() as i32),
        );
    }
    d.updating.set(false);
    let _ = InvalidateRect(Some(hwnd), None, false);
    let _ = InvalidateRect(Some(child(hwnd, PREVIEW)), None, false);
}
pub(super) unsafe fn changed(hwnd: HWND, d: &Data, id: i32, event: u16) {
    if d.updating.get() {
        return;
    }
    // EDIT 会先发 EN_UPDATE，不能在 EN_CHANGE 前把新文本覆盖成旧值。
    if ![EN_CHANGE as u16, EN_SETFOCUS as u16, EN_KILLFOCUS as u16].contains(&event) {
        return;
    }
    if id == HEX {
        if event == EN_SETFOCUS as u16 {
            d.hex_editing.set(true);
        }
        if event == EN_KILLFOCUS as u16 {
            d.hex_editing.set(false);
        }
        if event == EN_CHANGE as u16 {
            if let Some(color) = Rgba::parse(&text(child(hwnd, HEX))) {
                let mut v = d.value.get();
                v.load(color);
                d.value.set(v);
            }
        }
    } else if id == PERCENT && event == EN_CHANGE as u16 {
        if let Ok(percent) = text(child(hwnd, PERCENT)).parse::<f32>() {
            if percent.is_finite() {
                let mut v = d.value.get();
                v.a = percent.clamp(0.0, 100.0) / 100.0;
                d.value.set(v);
            }
        }
    }
    refresh(hwnd, d);
}
