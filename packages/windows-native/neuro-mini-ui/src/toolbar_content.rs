//! 首次使用的事务式分配；只在完整构建后发布，部分子控件和提示窗随回滚释放。
use super::*;
struct Pending {
    tip: Option<tooltip::Tooltip>,
    windows: Vec<HWND>,
    controls: Vec<Control>,
    committed: bool,
}
impl Drop for Pending {
    fn drop(&mut self) {
        // tooltip 保留文字指针；先移除其子类，再销毁按钮，最后才释放 Control 文字。
        drop(self.tip.take());
        if !self.committed {
            for hwnd in &self.windows {
                unsafe {
                    let _ = DestroyWindow(*hwnd);
                }
            }
        }
    }
}
struct Building<'a>(&'a Cell<bool>);
impl Drop for Building<'_> {
    fn drop(&mut self) {
        self.0.set(false);
    }
}
pub(super) unsafe fn prepare(hwnd: HWND) -> Result<()> {
    prepare_with(hwnd, |_| Ok(()))
}
#[cfg(test)]
#[path = "toolbar_content_tests.rs"]
mod tests;
pub(super) unsafe fn prepare_with(
    hwnd: HWND,
    mut check: impl FnMut(usize) -> Result<()>,
) -> Result<()> {
    let data = ui(hwnd).ok_or_else(|| Error::new(E_HANDLE, "Floating toolbar no longer exists"))?;
    if data.content.borrow().is_some() {
        return Ok(());
    }
    if data.building.replace(true) {
        return Err(Error::new(
            E_UNEXPECTED,
            "Floating toolbar preparation reentered",
        ));
    }
    let _building = Building(&data.building);
    let painter = Painter::new()?;
    let instance = HINSTANCE(GetModuleHandleW(None)?.0);
    let mut pending = Pending {
        tip: Some(tooltip::Tooltip::new(hwnd)?),
        windows: Vec::new(),
        controls: Vec::new(),
        committed: false,
    };
    for spec in data.items.iter().filter_map(|item| slots::spec(*item)) {
        check(pending.controls.len())?;
        let child = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            w!("BUTTON"),
            &HSTRING::from(spec.label),
            WS_CHILD | WS_TABSTOP | WINDOW_STYLE(BS_OWNERDRAW as u32),
            0,
            0,
            1,
            1,
            Some(hwnd),
            Some(HMENU(spec.id as usize as *mut _)),
            Some(instance),
            None,
        )?;
        pending.windows.push(child);
        crate::button::install(child)?;
        if spec.dropdown {
            crate::button_hold::install(child, data.dpi.get())?;
        }
        let label = HSTRING::from(spec.label);
        let corner = if spec.dropdown {
            let corner = CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                w!("BUTTON"),
                &HSTRING::from(format!("展开{}列表", spec.label)),
                WS_CHILD | WS_TABSTOP | WINDOW_STYLE(BS_OWNERDRAW as u32),
                0,
                0,
                1,
                1,
                Some(hwnd),
                Some(HMENU((spec.id | 0x8000) as usize as *mut _)),
                Some(instance),
                None,
            )?;
            pending.windows.push(corner);
            crate::button::install(corner)?;
            corner
        } else {
            HWND::default()
        };
        pending.controls.push(Control {
            spec,
            hwnd: child,
            corner,
            icon: Cell::new(spec.icon),
            active: Cell::new(false),
            label: RefCell::new(label),
            text: RefCell::new(None),
            visible: Cell::new(
                !data
                    .items
                    .iter()
                    .any(|item| matches!(item, Item::Slot(id) if *id == spec.id)),
            ),
            dropdown: Cell::new(spec.dropdown),
            context: Cell::new(0),
        });
        pending.tip.as_ref().unwrap().add(
            hwnd,
            child,
            &pending.controls.last().unwrap().label.borrow(),
        )?;
    }
    // 未发布状态时也可布局；SetWindowPos 重入看不到半套 Content。
    arrange_controls(&pending.controls, &data.layout.borrow())?;
    round(hwnd, &data.layout.borrow(), data.dpi.get())?;
    *data.content.borrow_mut() = Some(Content {
        painter,
        tip: pending.tip.take().unwrap(),
        controls: std::mem::take(&mut pending.controls),
        slot_font: RefCell::new(None),
    });
    pending.committed = true;
    Ok(())
}
pub(super) unsafe fn arrange(hwnd: HWND, layout: &Layout, dpi: u32) -> Result<()> {
    let Some(data) = ui(hwnd) else { return Ok(()) };
    if let Some(content) = data.content.borrow().as_ref() {
        arrange_controls(&content.controls, layout)?;
    }
    round(hwnd, layout, dpi)
}
unsafe fn arrange_controls(controls: &[Control], layout: &Layout) -> Result<()> {
    for control in controls.iter().filter(|control| !control.visible.get()) {
        let _ = ShowWindow(control.hwnd, SW_HIDE);
        let _ = ShowWindow(control.corner, SW_HIDE);
    }
    for (control, frame) in controls
        .iter()
        .filter(|control| control.visible.get())
        .zip(&layout.slots)
    {
        SetWindowPos(
            control.hwnd,
            None,
            frame.x,
            frame.y,
            frame.width,
            frame.height,
            SWP_NOZORDER | SWP_NOACTIVATE | SWP_SHOWWINDOW,
        )?;
        if !control.corner.is_invalid() && control.dropdown.get() {
            let size = (frame.width * 12 + 14) / 28;
            SetWindowPos(
                control.corner,
                Some(HWND_TOP),
                frame.x + frame.width - size,
                frame.y + frame.height - size,
                size,
                size,
                SWP_NOACTIVATE | SWP_SHOWWINDOW,
            )?;
        } else if !control.corner.is_invalid() {
            let _ = ShowWindow(control.corner, SW_HIDE);
        }
    }
    Ok(())
}
unsafe fn round(hwnd: HWND, layout: &Layout, dpi: u32) -> Result<()> {
    crate::strip_shell::region(
        hwnd,
        layout.width,
        layout.height,
        dpi,
        ui(hwnd).map_or(crate::strip_shell::Join::default(), |d| d.join.get()),
    )
}
