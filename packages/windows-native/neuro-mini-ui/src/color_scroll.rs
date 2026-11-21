//! 小工作区只滚动弹层内容，不缩放字号；键盘焦点始终被滚入可见区域。
use super::*;
pub(super) unsafe fn configure(hwnd: HWND, d: &Data) -> Result<()> {
    let mut client = RECT::default();
    // 一条滚动条会缩小另一维；先显示需要的轴，再据真实客户区计算 page。
    for _ in 0..2 {
        GetClientRect(hwnd, &mut client)?;
        let _ = ShowScrollBar(
            hwnd,
            SB_HORZ,
            client.right < crate::layout::px(geometry::WIDTH, d.dpi),
        );
        let _ = ShowScrollBar(
            hwnd,
            SB_VERT,
            client.bottom < crate::layout::px(geometry::HEIGHT, d.dpi),
        );
    }
    GetClientRect(hwnd, &mut client)?;
    for (bar, content, page, pos) in [
        (SB_HORZ, geometry::WIDTH, client.right, d.origin.get().x),
        (SB_VERT, geometry::HEIGHT, client.bottom, d.origin.get().y),
    ] {
        SetScrollInfo(
            hwnd,
            bar,
            &SCROLLINFO {
                cbSize: std::mem::size_of::<SCROLLINFO>() as u32,
                fMask: SIF_RANGE | SIF_PAGE | SIF_POS,
                nMin: 0,
                nMax: crate::layout::px(content, d.dpi) - 1,
                nPage: page.max(1) as u32,
                nPos: pos,
                ..Default::default()
            },
            true,
        );
    }
    Ok(())
}
unsafe fn move_to(hwnd: HWND, d: &Data, bar: SCROLLBAR_CONSTANTS, value: i32) {
    let old = d.origin.get();
    SetScrollPos(hwnd, bar, value, true);
    let value = GetScrollPos(hwnd, bar);
    let next = if bar == SB_VERT {
        POINT { x: old.x, y: value }
    } else {
        POINT { x: value, y: old.y }
    };
    if old == next {
        return;
    }
    d.origin.set(next);
    for id in controls::HEX..=controls::PALETTE {
        let child = controls::child(hwnd, id);
        let mut rect = RECT::default();
        if GetWindowRect(child, &mut rect).is_ok() {
            let mut point = POINT {
                x: rect.left,
                y: rect.top,
            };
            if ScreenToClient(hwnd, &mut point).as_bool() {
                let _ = SetWindowPos(
                    child,
                    None,
                    point.x + old.x - next.x,
                    point.y + old.y - next.y,
                    0,
                    0,
                    SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
                );
            }
        }
    }
    let _ = InvalidateRect(Some(hwnd), None, false);
}
pub(super) unsafe fn command(hwnd: HWND, d: &Data, vertical: bool, wp: WPARAM) {
    let bar = if vertical { SB_VERT } else { SB_HORZ };
    let mut info = SCROLLINFO {
        cbSize: std::mem::size_of::<SCROLLINFO>() as u32,
        fMask: SIF_ALL,
        ..Default::default()
    };
    if GetScrollInfo(hwnd, bar, &mut info).is_err() {
        return;
    }
    let line = crate::layout::px(24, d.dpi);
    let next = match SCROLLBAR_COMMAND(wp.0 as u16 as i32) {
        SB_LINEUP => info.nPos - line,
        SB_LINEDOWN => info.nPos + line,
        SB_PAGEUP => info.nPos - info.nPage as i32,
        SB_PAGEDOWN => info.nPos + info.nPage as i32,
        SB_TOP => 0,
        SB_BOTTOM => info.nMax,
        SB_THUMBTRACK | SB_THUMBPOSITION => info.nTrackPos,
        _ => return,
    };
    move_to(hwnd, d, bar, next);
}
pub(super) unsafe fn wheel(hwnd: HWND, d: &Data, delta: i32) {
    move_to(
        hwnd,
        d,
        SB_VERT,
        d.origin.get().y - delta * crate::layout::px(48, d.dpi) / 120,
    );
}
pub(super) unsafe fn reveal(hwnd: HWND, d: &Data) {
    let focus = GetFocus();
    if !IsChild(hwnd, focus).as_bool() {
        return;
    }
    let mut rect = RECT::default();
    let mut client = RECT::default();
    if GetWindowRect(focus, &mut rect).is_err() || GetClientRect(hwnd, &mut client).is_err() {
        return;
    }
    let mut top = POINT {
        x: rect.left,
        y: rect.top,
    };
    if !ScreenToClient(hwnd, &mut top).as_bool() {
        return;
    }
    for (bar, start, end, page, offset) in [
        (
            SB_HORZ,
            top.x,
            top.x + rect.right - rect.left,
            client.right,
            d.origin.get().x,
        ),
        (
            SB_VERT,
            top.y,
            top.y + rect.bottom - rect.top,
            client.bottom,
            d.origin.get().y,
        ),
    ] {
        if start < 0 {
            move_to(hwnd, d, bar, offset + start);
        } else if end > page {
            move_to(hwnd, d, bar, offset + end - page);
        }
    }
}
