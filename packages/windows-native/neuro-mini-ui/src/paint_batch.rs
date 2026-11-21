//! 原生控件树的绘制发布：保留旧画面直至数据和几何全部就绪，不引入动画或延后队列。
use windows::{
    core::{Error, Result},
    Win32::{Foundation::*, Graphics::Gdi::*, UI::WindowsAndMessaging::*},
};

/// WM_SETREDRAW 只冻结重绘，不用 ShowWindow 隐藏已提交画面。只对原本可见的根恢复可见位。
pub struct Hold {
    hwnd: HWND,
    identity: isize,
    resume: bool,
}
impl Hold {
    /// # Safety
    /// HWND belongs to this UI thread. Do not show/hide it until this guard is dropped.
    pub unsafe fn new(hwnd: HWND) -> Self {
        let identity = GetWindowLongPtrW(hwnd, GWLP_USERDATA);
        let resume = IsWindowVisible(hwnd).as_bool();
        if resume {
            SendMessageW(hwnd, WM_SETREDRAW, Some(WPARAM(0)), None);
        }
        Self {
            hwnd,
            identity,
            resume,
        }
    }
}
impl Drop for Hold {
    fn drop(&mut self) {
        unsafe {
            // 同步消息可重入；父先销毁后不得绘制复用同一数值的后继 HWND。
            if !IsWindow(Some(self.hwnd)).as_bool()
                || GetWindowLongPtrW(self.hwnd, GWLP_USERDATA) != self.identity
            {
                return;
            }
            if self.resume {
                SendMessageW(self.hwnd, WM_SETREDRAW, Some(WPARAM(1)), None);
            }
            if let Err(error) = present(self.hwnd) {
                eprintln!("neuro-mini-paint-batch: {error}");
            }
        }
    }
}
/// # Safety
/// HWND and its child tree belong to the calling UI thread.
pub unsafe fn present(hwnd: HWND) -> Result<()> {
    RedrawWindow(
        Some(hwnd),
        None,
        None,
        RDW_INVALIDATE | RDW_ALLCHILDREN | RDW_UPDATENOW,
    )
    .ok()
}
/// 标准 BUTTON 擦底也必须使用主题色；DC_BRUSH 是系统 stock object，不转移或释放所有权。
pub(crate) unsafe fn background(dc: HDC, rgb: u32) -> LRESULT {
    let colour = COLORREF(((rgb & 255) << 16) | (rgb & 0xff00) | ((rgb >> 16) & 255));
    SetBkColor(dc, colour);
    SetDCBrushColor(dc, colour);
    LRESULT(GetStockObject(DC_BRUSH).0 as isize)
}

struct Frame {
    dc: HDC,
    bitmap: HBITMAP,
    old: HGDIOBJ,
}
impl Drop for Frame {
    fn drop(&mut self) {
        unsafe {
            SelectObject(self.dc, self.old);
            let _ = DeleteObject(HGDIOBJ(self.bitmap.0));
            let _ = DeleteDC(self.dc);
        }
    }
}
impl Frame {
    unsafe fn new(destination: HDC, bounds: RECT) -> Result<Self> {
        let dc = CreateCompatibleDC(Some(destination));
        if dc.is_invalid() {
            return Err(Error::from_win32());
        }
        let bitmap = CreateCompatibleBitmap(destination, bounds.right.max(1), bounds.bottom.max(1));
        if bitmap.is_invalid() {
            let error = Error::from_win32();
            let _ = DeleteDC(dc);
            return Err(error);
        }
        let old = SelectObject(dc, HGDIOBJ(bitmap.0));
        if old.is_invalid() {
            let error = Error::from_win32();
            let _ = DeleteObject(HGDIOBJ(bitmap.0));
            let _ = DeleteDC(dc);
            return Err(error);
        }
        Ok(Self { dc, bitmap, old })
    }
}
/// 成功前目标 DC 字节不变；D2D 的 EndDraw 和之后 GDI 文字不能分别暴露到屏幕。
pub(crate) unsafe fn frame(
    dc: HDC,
    bounds: RECT,
    draw: impl FnOnce(HDC) -> Result<()>,
) -> Result<()> {
    let buffer = Frame::new(dc, bounds)?;
    draw(buffer.dc)?;
    BitBlt(
        dc,
        bounds.left,
        bounds.top,
        bounds.right - bounds.left,
        bounds.bottom - bounds.top,
        Some(buffer.dc),
        bounds.left,
        bounds.top,
        SRCCOPY,
    )
}
/// # Safety
/// The DC and bounds are valid for the duration of draw; draw must restore selected objects.
pub unsafe fn control(dc: HDC, bounds: RECT, draw: impl FnOnce(HDC) -> Result<()>) -> Result<()> {
    if GetObjectType(HGDIOBJ(dc.0)) == OBJ_MEMDC.0 as u32 {
        // WM_PRINT 已在根离屏帧内，不再为每个图标或步进重复分配位图。
        draw(dc)
    } else {
        frame(dc, bounds, draw)
    }
}
/// # Safety
/// HWND belongs to this UI thread and dc is its current paint/print destination.
pub unsafe fn window(hwnd: HWND, dc: HDC, draw: impl FnOnce(HDC) -> Result<()>) -> Result<()> {
    let mut bounds = RECT::default();
    GetClientRect(hwnd, &mut bounds)?;
    frame(dc, bounds, |out| {
        draw(out)?;
        // 按真实 sibling 层序从底向上打印完整 child 树，保留标准 EDIT 草稿、选区和系统键盘。
        // 不用 WS_EX_COMPOSITED：它在当前 Windows 上会使捕获排除设置返回 ERROR_NOT_ENOUGH_MEMORY。
        let first = GetWindow(hwnd, GW_CHILD).ok();
        let mut child = first.and_then(|h| GetWindow(h, GW_HWNDLAST).ok());
        while let Some(h) = child {
            if GetWindowLongW(h, GWL_STYLE) as u32 & WS_VISIBLE.0 != 0 {
                let mut origin = POINT::default();
                MapWindowPoints(Some(h), Some(hwnd), std::slice::from_mut(&mut origin));
                let mut client = RECT::default();
                GetClientRect(h, &mut client)?;
                let saved = SaveDC(out);
                if saved == 0 {
                    return Err(Error::from_win32());
                }
                IntersectClipRect(
                    out,
                    origin.x,
                    origin.y,
                    origin.x + client.right,
                    origin.y + client.bottom,
                );
                SetViewportOrgEx(out, origin.x, origin.y, None).ok()?;
                SendMessageW(
                    h,
                    WM_PRINT,
                    Some(WPARAM(out.0 as usize)),
                    Some(LPARAM(
                        (PRF_CLIENT | PRF_CHILDREN | PRF_ERASEBKGND) as isize,
                    )),
                );
                RestoreDC(out, saved).ok()?;
            }
            child = GetWindow(h, GW_HWNDPREV).ok();
        }
        Ok(())
    })
}
#[cfg(test)]
#[path = "paint_batch_tests.rs"]
mod tests;
