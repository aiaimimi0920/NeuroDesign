//! 菜单外壳和真实按钮；无激活 HWND 不改变钉图的全局置顶关系。
use super::*;
pub(super) unsafe fn region(popup: &Popup) -> Result<()> {
    let d = &popup.data;
    let px = |v| crate::layout::px(v, d.dpi);
    let top = if d.above { 0 } else { px(6) };
    let bottom = d.height - if d.above { px(6) } else { 0 };
    let round = CreateRoundRectRgn(
        0,
        top,
        d.width + 1,
        bottom + 1,
        px(if d.quiet { 12 } else { 8 }),
        px(if d.quiet { 12 } else { 8 }),
    );
    let y = if d.above { bottom - 1 } else { top + 1 };
    let tip = if d.above { d.height } else { 0 };
    let triangle = CreatePolygonRgn(
        &[
            POINT {
                x: d.pointer - px(5),
                y,
            },
            POINT {
                x: d.pointer,
                y: tip,
            },
            POINT {
                x: d.pointer + px(5),
                y,
            },
        ],
        WINDING,
    );
    if round.is_invalid() || triangle.is_invalid() {
        if !round.is_invalid() {
            let _ = DeleteObject(HGDIOBJ(round.0)).ok();
        }
        if !triangle.is_invalid() {
            let _ = DeleteObject(HGDIOBJ(triangle.0)).ok();
        }
        return Err(Error::from_win32());
    }
    let combined = CombineRgn(Some(round), Some(round), Some(triangle), RGN_OR);
    let _ = DeleteObject(HGDIOBJ(triangle.0)).ok();
    if combined == RGN_ERROR || SetWindowRgn(popup.hwnd, Some(round), true) == 0 {
        let _ = DeleteObject(HGDIOBJ(round.0)).ok();
        return Err(Error::from_win32());
    }
    Ok(())
}
pub(super) unsafe extern "system" fn procedure(
    hwnd: HWND,
    msg: u32,
    wp: WPARAM,
    lp: LPARAM,
) -> LRESULT {
    std::panic::catch_unwind(|| route(hwnd, msg, wp, lp)).unwrap_or(LRESULT(0))
}
unsafe fn route(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    let Some(data) = (GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const Data).as_ref() else {
        return DefWindowProcW(hwnd, msg, wp, lp);
    };
    match msg {
        WM_MOUSEACTIVATE => LRESULT(MA_NOACTIVATE as isize),
        WM_ERASEBKGND => LRESULT(1),
        WM_PAINT => {
            let mut p = PAINTSTRUCT::default();
            let dc = BeginPaint(hwnd, &mut p);
            if let Err(e) = paint::shell(data, dc) {
                eprintln!("neuro-mini-menu-shell: {e}");
            }
            let _ = EndPaint(hwnd, &p).ok();
            LRESULT(0)
        }
        WM_DRAWITEM if lp.0 != 0 => {
            let item = &*(lp.0 as *const DRAWITEMSTRUCT);
            if let Some(index) = item
                .CtlID
                .checked_sub(1)
                .map(|v| v as usize)
                .filter(|i| *i < data.items.len())
            {
                if let Err(e) = paint::row(data, index, item) {
                    eprintln!("neuro-mini-menu-row: {e}");
                }
            }
            LRESULT(1)
        }
        WM_COMMAND if wp.0 >> 16 == BN_CLICKED as usize => {
            let id = (wp.0 & 0xffff).saturating_sub(1);
            if let Some(item) = data.items.get(id).filter(|i| i.enabled) {
                data.command.set(Some(item.id));
                data.closed.set(true);
            }
            LRESULT(0)
        }
        WM_CLOSE | WM_CANCELMODE => {
            data.closed.set(true);
            LRESULT(0)
        }
        WM_NCDESTROY => {
            data.closed.set(true);
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
            DefWindowProcW(hwnd, msg, wp, lp)
        }
        _ => DefWindowProcW(hwnd, msg, wp, lp),
    }
}
