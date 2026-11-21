//! 窄面板的有界像素滚动；滚轮/轨道立即投影，不改变值、不创建动画或任务。
use super::*;
pub(super) fn maximum(d: &Data) -> f64 {
    if d.stack_width.is_none() {
        return 0.0;
    }
    (d.content_height.get() - d.height.get() + crate::layout::px(8, d.dpi.get())).max(0) as f64
}
pub(super) fn track(d: &Data) -> RECT {
    let px = |v| crate::layout::px(v, d.dpi.get());
    RECT {
        left: d.width.get() - px(12),
        right: d.width.get() - px(4),
        top: px(4),
        bottom: d.height.get() - px(4),
    }
}
pub(super) fn thumb(d: &Data) -> RECT {
    let mut r = track(d);
    let height = r.bottom - r.top;
    let size = ((height as f64 * height as f64 / d.content_height.get().max(1) as f64).round()
        as i32)
        .max(crate::layout::px(18, d.dpi.get()))
        .min(height);
    r.top += ((height - size) as f64 * d.scroll_top.get() / maximum(d).max(1.0)).round() as i32;
    r.bottom = r.top + size;
    r
}
pub(super) unsafe fn set(d: &Data, top: f64) -> Result<()> {
    let top = top.clamp(0.0, maximum(d));
    if d.scroll_top.replace(top) != top {
        layout::project(d)?;
    }
    Ok(())
}
pub(super) unsafe fn reveal(d: &Data, hwnd: HWND) -> Result<()> {
    if d.stack_width.is_none() {
        return Ok(());
    }
    let frame = d
        .controls
        .borrow()
        .iter()
        .find(|c| c.hwnd == hwnd || c.step.contains(&hwnd))
        .map(|c| c.frame);
    if let Some(r) = frame {
        let height = d.height.get() - crate::layout::px(8, d.dpi.get());
        if (r.top as f64) < d.scroll_top.get() {
            set(d, r.top as f64)?;
        } else if r.bottom as f64 > d.scroll_top.get() + height as f64 {
            set(d, (r.bottom - height) as f64)?;
        }
    }
    Ok(())
}
pub(super) unsafe fn wheel(d: &Data, wp: WPARAM) -> Result<()> {
    set(
        d,
        d.scroll_top.get()
            - (wp.0 >> 16) as i16 as f64 * crate::layout::px(30, d.dpi.get()) as f64 / 120.0,
    )
}
unsafe fn cancel(hwnd: HWND, d: &Data) {
    if d.scroll_drag.replace(None).is_some() && GetCapture() == hwnd {
        let _ = ReleaseCapture();
    }
}
pub(super) unsafe fn handle(
    hwnd: HWND,
    d: &Data,
    msg: u32,
    wp: WPARAM,
    lp: LPARAM,
) -> Result<Option<LRESULT>> {
    if d.stack_width.is_none() {
        return Ok(None);
    }
    let point = POINT {
        x: lp.0 as i16 as i32,
        y: (lp.0 >> 16) as i16 as i32,
    };
    match msg {
        WM_MOUSEWHEEL => wheel(d, wp)?,
        WM_LBUTTONDOWN if hwnd != d.viewport.get() && maximum(d) > 0.0 => {
            let r = track(d);
            if point.x < r.left || point.x >= r.right || point.y < r.top || point.y >= r.bottom {
                return Ok(None);
            }
            let _ = SetFocus(Some(hwnd));
            let t = thumb(d);
            if point.y < t.top || point.y >= t.bottom {
                set(
                    d,
                    d.scroll_top.get()
                        + if point.y < t.top { -1.0 } else { 1.0 } * (r.bottom - r.top) as f64,
                )?;
            } else {
                d.scroll_drag.set(Some((point.y, d.scroll_top.get())));
                let _ = SetCapture(hwnd);
            }
        }
        WM_MOUSEMOVE if d.scroll_drag.get().is_some() => {
            let (y, top) = d.scroll_drag.get().unwrap();
            let r = track(d);
            let t = thumb(d);
            let travel = r.bottom - r.top - (t.bottom - t.top);
            if travel > 0 {
                set(d, top + (point.y - y) as f64 * maximum(d) / travel as f64)?;
            }
        }
        WM_LBUTTONUP if d.scroll_drag.get().is_some() => {
            d.scroll_drag.set(None);
            if GetCapture() == hwnd {
                let _ = ReleaseCapture();
            }
        }
        WM_CAPTURECHANGED | WM_CANCELMODE if hwnd != d.viewport.get() => {
            cancel(hwnd, d);
            return Ok(None);
        }
        WM_SHOWWINDOW if wp.0 == 0 && hwnd != d.viewport.get() => {
            // Win32 隐藏窗口不会自动交还捕获；只释放这次轨道拖动自己的租约。
            cancel(hwnd, d);
            return Ok(None);
        }
        WM_SETFOCUS | WM_KILLFOCUS => {
            let _ = InvalidateRect(Some(hwnd), None, false);
            return Ok(None);
        }
        WM_KEYDOWN if GetFocus() == hwnd && hwnd != d.viewport.get() => {
            let top = d.scroll_top.get();
            let row = crate::layout::px(30, d.dpi.get()) as f64;
            let page = (d.height.get() - crate::layout::px(8, d.dpi.get())) as f64;
            let next = match wp.0 as u16 {
                k if k == VK_UP.0 => top - row,
                k if k == VK_DOWN.0 => top + row,
                k if k == VK_PRIOR.0 => top - page,
                k if k == VK_NEXT.0 => top + page,
                k if k == VK_HOME.0 => 0.0,
                k if k == VK_END.0 => maximum(d),
                _ => return Ok(None),
            };
            set(d, next)?;
        }
        _ => return Ok(None),
    }
    Ok(Some(LRESULT(0)))
}
pub(super) unsafe fn paint(d: &Data, dc: HDC) -> Result<()> {
    if maximum(d) == 0.0 {
        return Ok(());
    }
    let brush = CreateSolidBrush(super::paint::rgb(crate::paint::mix(
        d.painter.palette.surface,
        d.painter.palette.text,
        0.24,
    )));
    if brush.is_invalid() {
        return Err(Error::from_win32());
    }
    FillRect(dc, &thumb(d), brush);
    let _ = DeleteObject(HGDIOBJ(brush.0));
    if GetFocus() == GetParent(d.viewport.get()).unwrap_or_default() {
        let focus = CreateSolidBrush(super::paint::rgb(d.painter.palette.signal));
        if focus.is_invalid() {
            return Err(Error::from_win32());
        }
        FrameRect(dc, &thumb(d), focus);
        let _ = DeleteObject(HGDIOBJ(focus.0));
    }
    Ok(())
}
