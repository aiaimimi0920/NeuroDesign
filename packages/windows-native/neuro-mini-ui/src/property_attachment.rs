//! 二级行的零间隙几何、接边和纯移动投影；宿主可把这组位置放入自己的原子移动批次。
use super::*;

fn planned(anchor: RECT, work: RECT, width: i32, height: i32, dpi: u32) -> RECT {
    let margin = crate::layout::px(4, dpi);
    let left = anchor.left.clamp(
        work.left + margin,
        (work.right - margin - width).max(work.left + margin),
    );
    let top = if anchor.top - height >= work.top + margin {
        anchor.top - height
    } else {
        anchor
            .bottom
            .min(work.bottom - margin - height)
            .max(work.top + margin)
    };
    RECT {
        left,
        top,
        right: left + width,
        bottom: top + height,
    }
}
unsafe fn target(bar: &Bar, anchor: RECT) -> Result<RECT> {
    let mut info = MONITORINFO {
        cbSize: std::mem::size_of::<MONITORINFO>() as u32,
        ..Default::default()
    };
    if !GetMonitorInfoW(
        MonitorFromRect(&anchor, MONITOR_DEFAULTTONEAREST),
        &mut info,
    )
    .as_bool()
    {
        return Err(Error::from_win32());
    }
    Ok(planned(
        anchor,
        info.rcWork,
        bar.data.width.get(),
        bar.data.height.get(),
        bar.data.dpi.get(),
    ))
}
unsafe fn main_join(toolbar: HWND, join: crate::strip_shell::Join) -> Result<()> {
    let ui =
        toolbar::ui(toolbar).ok_or_else(|| Error::new(E_HANDLE, "Toolbar no longer exists"))?;
    if ui.join.get() != join {
        let mut rect = RECT::default();
        GetClientRect(toolbar, &mut rect)?;
        crate::strip_shell::region(
            toolbar,
            rect.right,
            rect.bottom,
            GetDpiForWindow(toolbar).max(96),
            join,
        )?;
        ui.join.set(join);
    }
    Ok(())
}
pub(super) unsafe fn clear(toolbar: HWND) -> Result<()> {
    main_join(toolbar, crate::strip_shell::Join::default())
}
pub(super) unsafe fn place(bar: &Bar) -> Result<()> {
    let d = &bar.data;
    let mut anchor = RECT::default();
    GetWindowRect(d.toolbar, &mut anchor)?;
    let next = target(bar, anchor)?;
    let join = if d.stack_width.is_none() && d.active.get() {
        main_join(d.toolbar, crate::strip_shell::Join::between(anchor, next))?;
        crate::strip_shell::Join::between(next, anchor)
    } else {
        crate::strip_shell::Join::default()
    };
    let mut old = RECT::default();
    GetWindowRect(bar.hwnd, &mut old)?;
    if old.right - old.left != next.right - next.left
        || old.bottom - old.top != next.bottom - next.top
        || d.join.get() != join
    {
        crate::strip_shell::region(bar.hwnd, d.width.get(), d.height.get(), d.dpi.get(), join)?;
        d.join.set(join);
    }
    if old != next {
        SetWindowPos(
            bar.hwnd,
            None,
            next.left,
            next.top,
            d.width.get(),
            d.height.get(),
            SWP_NOZORDER | SWP_NOACTIVATE | SWP_NOOWNERZORDER,
        )?;
    }
    Ok(())
}
/// 返回已显示二级行相对主条目标矩形的位置；不移动窗口，不复制宿主的业务附着策略。
/// # Safety
/// 同 create；只在 UI 线程使用。调用方应在同一批次移动主条及返回窗口。
pub unsafe fn attached_position(hwnd: HWND, proposed: RECT) -> Result<Option<(HWND, RECT)>> {
    let ui = toolbar::ui(hwnd).ok_or_else(|| Error::new(E_HANDLE, "Toolbar no longer exists"))?;
    let bars = ui.properties.borrow();
    let Some(bar) = bars
        .as_ref()
        .filter(|bar| bar.alive() && IsWindowVisible(bar.hwnd).as_bool())
    else {
        return Ok(None);
    };
    Ok(Some((bar.hwnd, target(bar, proposed)?)))
}
/// 完成宿主移动后的接边/region 同步；位置相同不再次移动或重绘。
/// # Safety
/// 同 create；必须在同一 UI 线程调用。
pub unsafe fn sync_attached(hwnd: HWND) -> Result<()> {
    let ui = toolbar::ui(hwnd).ok_or_else(|| Error::new(E_HANDLE, "Toolbar no longer exists"))?;
    if let Some(bar) = ui.properties.borrow().as_ref() {
        bar.place()?;
    }
    Ok(())
}
#[cfg(test)]
#[path = "property_attachment_tests.rs"]
mod tests;
