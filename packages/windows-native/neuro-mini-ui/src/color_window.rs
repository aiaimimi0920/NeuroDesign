//! 窗口过程只操作弹层状态；失败关闭并传回错误，销毁/失捕获都会结束短暂拖动。
use super::*;
#[path = "color_clipboard.rs"]
mod clipboard;
pub(super) unsafe extern "system" fn procedure(
    hwnd: HWND,
    msg: u32,
    wp: WPARAM,
    lp: LPARAM,
) -> LRESULT {
    let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const Data;
    if ptr.is_null() {
        return DefWindowProcW(hwnd, msg, wp, lp);
    }
    let d = &*ptr;
    let result = handle(hwnd, d, msg, wp, lp);
    match result {
        Ok(Some(value)) => value,
        Ok(None) => DefWindowProcW(hwnd, msg, wp, lp),
        Err(error) => {
            *d.error.borrow_mut() = Some(error);
            d.closed.set(true);
            LRESULT(0)
        }
    }
}
unsafe fn handle(
    hwnd: HWND,
    d: &Data,
    msg: u32,
    wp: WPARAM,
    lp: LPARAM,
) -> Result<Option<LRESULT>> {
    match msg {
        WM_PAINT => {
            let mut ps = PAINTSTRUCT::default();
            let dc = BeginPaint(hwnd, &mut ps);
            let result = paint::shell(d, dc);
            let _ = EndPaint(hwnd, &ps);
            result?;
        }
        WM_ERASEBKGND => return Ok(Some(LRESULT(1))),
        WM_VSCROLL | WM_HSCROLL => scroll::command(hwnd, d, msg == WM_VSCROLL, wp),
        WM_DRAWITEM => {
            if lp.0 != 0 {
                paint::control(d, &*(lp.0 as *const DRAWITEMSTRUCT))?;
            }
        }
        WM_CTLCOLOREDIT => {
            let dc = HDC(wp.0 as *mut _);
            SetTextColor(dc, paint::rgb(d.painter.palette.text));
            SetBkColor(dc, paint::rgb(d.painter.palette.control));
            return Ok(Some(LRESULT(d.brush.0 as isize)));
        }
        WM_COMMAND => {
            let id = (wp.0 & 0xffff) as i32;
            let event = (wp.0 >> 16) as u16;
            if matches!(id, controls::HEX | controls::PERCENT) {
                controls::changed(hwnd, d, id, event);
            } else if event == BN_CLICKED as u16 {
                match id {
                    controls::CANCEL => d.closed.set(true),
                    controls::APPLY => {
                        d.action.set(ColorAction::Apply(d.value.get().color()));
                        d.closed.set(true);
                    }
                    controls::PICK if d.screen_pick => {
                        d.action.set(ColorAction::ScreenPick);
                        d.closed.set(true);
                    }
                    controls::PREVIEW => clipboard::copy(hwnd, &d.value.get().color().hex())?,
                    _ => (),
                }
            }
        }
        WM_CONTEXTMENU if HWND(wp.0 as *mut _) == controls::child(hwnd, controls::PREVIEW) => {
            clipboard::copy(hwnd, &d.value.get().color().rgb_text())?
        }
        WM_MOUSEMOVE if d.drag.get().is_some() => {
            let p = POINT {
                x: ((lp.0 as i16 as i32 + d.origin.get().x) as f32 * 96.0 / d.dpi as f32).floor()
                    as i32,
                y: (((lp.0 >> 16) as i16 as i32 + d.origin.get().y) as f32 * 96.0 / d.dpi as f32)
                    .floor() as i32,
            };
            input::pointer::sample(hwnd, d, p);
        }
        WM_LBUTTONUP => {
            d.drag.set(None);
            if GetCapture() == hwnd {
                let _ = ReleaseCapture();
            }
        }
        WM_CAPTURECHANGED => d.drag.set(None),
        WM_CLOSE | WM_CANCELMODE => d.closed.set(true),
        WM_NCDESTROY => {
            d.drag.set(None);
            d.closed.set(true);
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
            return Ok(None);
        }
        _ => return Ok(None),
    }
    Ok(Some(LRESULT(0)))
}
