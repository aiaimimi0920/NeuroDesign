//! 原生 EDIT 延迟提交；Enter/blur 提交，Escape 丢草稿，方向键按一单位步进。
use super::*;
use windows::Win32::UI::Shell::*;
const ID: usize = 0x4e51;
pub(super) unsafe fn install(hwnd: HWND) -> Result<()> {
    SetWindowSubclass(hwnd, Some(procedure), ID, 0).ok()
}
pub(super) unsafe fn commit(parent: HWND, child: HWND, step: i32, cancel: bool) {
    let Some(d) = (GetWindowLongPtrW(parent, GWLP_USERDATA) as *const Data).as_ref() else {
        return;
    };
    if d.updating.get() {
        return;
    }
    if ratio::commit(d, child, step, cancel) {
        return;
    }
    let mut fields = d.controls.borrow_mut();
    let Some(c) = fields.iter_mut().find(|c| c.hwnd == child) else {
        return;
    };
    let mut raw = [0u16; 65];
    let n = GetWindowTextW(child, &mut raw) as usize;
    let text = String::from_utf16_lossy(&raw[..n]);
    if cancel {
        c.dirty = false;
    }
    let value = match &mut c.field.kind {
        PropertyKind::Number {
            value, min, max, ..
        } => {
            if !cancel && (c.dirty || step != 0) {
                let parsed = if text.is_empty() {
                    None
                } else {
                    text.parse::<i32>().ok()
                };
                *value = parsed
                    .unwrap_or(*value)
                    .saturating_add(step)
                    .clamp(*min, *max);
            }
            value.to_string()
        }
        PropertyKind::Text { value, .. } => {
            if !cancel && c.dirty && !text.trim().is_empty() {
                *value = text;
            }
            value.clone()
        }
        _ => return,
    };
    let changed = !cancel && (c.dirty || step != 0);
    c.dirty = false;
    let first = c.pending.is_none();
    if changed {
        c.pending = Some(PropertyEdit {
            context: d.context.get(),
            id: c.field.id,
            value: Some(value.clone()),
            active: None,
        });
    }
    let id = c.field.id;
    drop(fields);
    d.updating.set(true);
    let _ = SetWindowTextW(child, &HSTRING::from(value));
    d.updating.set(false);
    if changed && first {
        notify(d, id);
    }
}
pub(super) unsafe fn notify(d: &Data, id: u16) {
    if let Some(ui) = toolbar::ui(d.toolbar) {
        let _ = PostMessageW(
            Some(ui.owner),
            ui.notify,
            WPARAM(id as usize),
            LPARAM(if d.stack_width.is_some() {
                PANEL
            } else {
                PROPERTY
            }),
        );
    }
}
unsafe extern "system" fn procedure(
    hwnd: HWND,
    msg: u32,
    wp: WPARAM,
    lp: LPARAM,
    _: usize,
    _: usize,
) -> LRESULT {
    match msg {
        WM_GETDLGCODE => return LRESULT((DLGC_WANTARROWS | DLGC_WANTCHARS) as isize),
        WM_KEYDOWN => {
            let key = wp.0 as u16;
            if matches!(key,k if k==VK_RETURN.0||k==VK_ESCAPE.0||k==VK_UP.0||k==VK_DOWN.0) {
                if let Ok(parent) = GetParent(hwnd) {
                    commit(
                        parent,
                        hwnd,
                        if key == VK_UP.0 {
                            1
                        } else if key == VK_DOWN.0 {
                            -1
                        } else {
                            0
                        },
                        key == VK_ESCAPE.0,
                    );
                    if key == VK_RETURN.0 || key == VK_ESCAPE.0 {
                        let _ = SetFocus(Some(parent));
                    }
                }
                return LRESULT(0);
            }
        }
        WM_CHAR if wp.0 == VK_RETURN.0 as usize || wp.0 == VK_ESCAPE.0 as usize => {
            return LRESULT(0)
        }
        WM_KILLFOCUS => {
            if let Ok(parent) = GetParent(hwnd) {
                commit(parent, hwnd, 0, false);
            }
        }
        WM_NCDESTROY => {
            let _ = RemoveWindowSubclass(hwnd, Some(procedure), ID);
        }
        _ => {}
    }
    DefSubclassProc(hwnd, msg, wp, lp)
}
