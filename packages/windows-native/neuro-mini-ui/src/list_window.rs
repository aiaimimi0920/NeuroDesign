//! 根弹层和裁剪 viewport 共用过程；只向消息泵返回选择结果，不执行宿主命令。
use super::*;
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
    match handle(hwnd, d, msg, wp, lp) {
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
            let result = crate::paint_batch::window(hwnd, dc, |out| paint::shell(d, hwnd, out));
            let _ = EndPaint(hwnd, &ps);
            result?;
        }
        WM_ERASEBKGND => return Ok(Some(LRESULT(1))),
        WM_PRINTCLIENT => paint::shell(d, hwnd, HDC(wp.0 as *mut _))?,
        WM_CTLCOLORBTN => {
            return Ok(Some(crate::paint_batch::background(
                HDC(wp.0 as *mut _),
                crate::paint::mix(0x0e1218, d.painter.palette.text, 0.04),
            )));
        }
        WM_WINDOWPOSCHANGED
            if lp.0 != 0 && (*(lp.0 as *const WINDOWPOS)).flags.contains(SWP_SHOWWINDOW) =>
        {
            let result = DefWindowProcW(hwnd, msg, wp, lp);
            crate::paint_batch::present(hwnd)?;
            return Ok(Some(result));
        }
        WM_MOUSEACTIVATE => return Ok(Some(LRESULT(MA_NOACTIVATE as isize))),
        WM_DRAWITEM if lp.0 != 0 => {
            let mut draw = *(lp.0 as *const DRAWITEMSTRUCT);
            crate::paint_batch::control(draw.hDC, draw.rcItem, |out| {
                draw.hDC = out;
                paint::row(d, &draw)
            })?;
        }
        WM_COMMAND if (wp.0 >> 16) as u16 == BN_CLICKED as u16 && lp.0 != 0 => {
            let index = GetWindowLongPtrW(HWND(lp.0 as *mut _), GWLP_USERDATA);
            if index > 0 {
                d.choose(index as usize - 1);
            }
        }
        WM_CLOSE | WM_CANCELMODE => d.closed.set(true),
        WM_CAPTURECHANGED if hwnd == d.root.get() && GetCapture() != hwnd => d.closed.set(true),
        WM_NCDESTROY => {
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
            if hwnd == d.root.get() {
                d.closed.set(true);
            }
            return Ok(None);
        }
        _ => return Ok(None),
    }
    Ok(Some(LRESULT(0)))
}
