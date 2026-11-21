//! 工具条所属的窄纵向属性面板；复用属性控件，不拥有应用的附着策略或业务字段。
use super::*;
pub const PANEL: isize = 5;
pub const PANEL_CLOSE: isize = 6;
pub struct PropertyPanel(Bar);
impl PropertyPanel {
    /// # Safety
    /// toolbar 必须是本 UI 线程创建且存活的 Neuro-mini 工具条。面板须在同一线程释放。
    pub unsafe fn new(toolbar: HWND, width: u16) -> Result<Self> {
        if !(80..=512).contains(&width) || toolbar::ui(toolbar).is_none() {
            return Err(Error::new(E_INVALIDARG, "Invalid attached property panel"));
        }
        Ok(Self(window::create_at(toolbar, Some(width))?))
    }
    /// # Safety
    /// 必须在创建线程调用；context 在业务目标换代时改变。
    pub unsafe fn update(
        &self,
        context: u64,
        fields: Vec<PropertyField>,
        enabled: bool,
    ) -> Result<()> {
        self.update_with_states(context, fields.into_iter().map(|f| (f, enabled)).collect())
    }
    /// 原子更新字段和值的最终可用性；相同状态不重绘，不产生先启用再禁用的中间帧。
    /// # Safety
    /// 同 update；字段和可用性必须来自同一业务快照。
    pub unsafe fn update_with_states(
        &self,
        context: u64,
        fields: Vec<(PropertyField, bool)>,
    ) -> Result<()> {
        self.update_sections(context, vec![fields])
    }
    /// 连续纵向分组；最多八组、总共三十二个字段，不创建额外面板或重建公共输入。
    /// # Safety
    /// 同 update；分组只决定布局，业务身份与状态仍由调用方提供。
    pub unsafe fn update_sections(
        &self,
        context: u64,
        sections: Vec<Vec<(PropertyField, bool)>>,
    ) -> Result<()> {
        if sections.len() > 8 {
            return Err(Error::new(E_INVALIDARG, "Too many property panel sections"));
        }
        let starts = sections
            .iter()
            .skip(1)
            .filter_map(|s| s.first().map(|(f, _)| f.id))
            .collect();
        let fields: Vec<_> = sections.into_iter().flatten().collect();
        validate(
            &fields.iter().map(|(f, _)| f.clone()).collect::<Vec<_>>(),
            32,
        )?;
        if !self.0.alive() {
            return Err(Error::new(E_HANDLE, "Property panel was destroyed"));
        }
        let redraw = controls::update(&self.0, context, fields, starts)?;
        drop(redraw);
        Ok(())
    }
    /// # Safety
    /// 同 update。返回物理像素，布局约束与放置策略由应用决定。
    pub unsafe fn size(&self) -> (i32, i32) {
        (self.0.data.width.get(), self.0.data.height.get())
    }
    /// 限制可见高度（屏幕物理像素）；溢出在内部滚动，不缩小字号或控件。
    /// # Safety
    /// 同 update；宿主在每次工作区约束改变后、读取 size/place 前调用。
    pub unsafe fn set_max_height(&self, height: i32) -> Result<()> {
        if height < crate::layout::px(36, self.0.data.dpi.get()) {
            return Err(Error::new(
                E_BOUNDS,
                "Property panel needs one readable row",
            ));
        }
        if !self.0.alive() {
            return Err(Error::new(E_HANDLE, "Property panel was destroyed"));
        }
        if self.0.data.max_height.replace(height) != height {
            layout::refresh(&self.0)?;
        }
        Ok(())
    }
    /// # Safety
    /// 同 update。宿主不得修改返回 HWND 的 userdata。
    pub unsafe fn window(&self) -> Option<HWND> {
        self.0.alive().then_some(self.0.hwnd)
    }
    /// 字段的屏幕矩形，用于附着宿主拥有的弹层。
    /// # Safety
    /// 同 update；失效/移除字段返回 None。
    pub unsafe fn anchor(&self, id: u16) -> Option<RECT> {
        self.window()?;
        let fields = self.0.data.controls.borrow();
        let field = fields.iter().find(|c| c.field.id == id)?;
        let mut rect = RECT::default();
        GetWindowRect(field.hwnd, &mut rect).ok()?;
        Some(rect)
    }
    /// # Safety
    /// 同 update；区别用户关闭请求和宿主暂时隐藏。
    pub unsafe fn is_open(&self) -> bool {
        self.0.data.active.get()
    }
    /// # Safety
    /// 同 update。
    pub unsafe fn show(&self, show: bool) {
        self.0.data.active.set(show);
        self.0.visible(show);
    }
    /// # Safety
    /// 同 update；rect 来自宿主已约束的屏幕物理坐标。
    pub unsafe fn place(&self, rect: RECT) -> Result<()> {
        if rect.right - rect.left != self.0.data.width.get()
            || rect.bottom - rect.top != self.0.data.height.get()
        {
            return Err(Error::new(E_INVALIDARG, "Property panel size mismatch"));
        }
        let hwnd = self
            .window()
            .ok_or_else(|| Error::new(E_HANDLE, "Property panel was destroyed"))?;
        let mut old = RECT::default();
        GetWindowRect(hwnd, &mut old)?;
        if old != rect {
            SetWindowPos(
                hwnd,
                None,
                rect.left,
                rect.top,
                rect.right - rect.left,
                rect.bottom - rect.top,
                SWP_NOZORDER | SWP_NOACTIVATE | SWP_NOOWNERZORDER,
            )?;
            let radius = crate::layout::px(18, self.0.data.dpi.get());
            let region = CreateRoundRectRgn(
                0,
                0,
                rect.right - rect.left + 1,
                rect.bottom - rect.top + 1,
                radius,
                radius,
            );
            if region.is_invalid() {
                return Err(Error::from_win32());
            }
            if SetWindowRgn(hwnd, Some(region), true) == 0 {
                let _ = DeleteObject(HGDIOBJ(region.0));
                return Err(Error::from_win32());
            }
        }
        Ok(())
    }
    /// # Safety
    /// 同 update。应用须复核提交的 context 与目标，不跨消息持有本对象借用。
    pub unsafe fn take_edit(&self, id: u16) -> Option<PropertyEdit> {
        self.0
            .data
            .controls
            .borrow_mut()
            .iter_mut()
            .find(|c| c.field.id == id)?
            .pending
            .take()
    }
    /// # Safety
    /// 同 update。
    pub unsafe fn contains(&self, hwnd: HWND) -> bool {
        self.0.alive() && (hwnd == self.0.hwnd || IsChild(self.0.hwnd, hwnd).as_bool())
    }
    /// # Safety
    /// 同 update；message 来自真实 UI 消息泵。
    pub unsafe fn translate(&self, message: &MSG) -> bool {
        super::translate(&self.0, message)
    }
    /// # Safety
    /// 同 update。单字段准入不会改变同面板其它控件的状态。
    pub unsafe fn enable(&self, id: u16, enabled: bool) {
        if let Some(c) = self
            .0
            .data
            .controls
            .borrow()
            .iter()
            .find(|c| c.field.id == id)
        {
            for h in c.handles() {
                let _ = EnableWindow(h, enabled);
            }
        }
    }
}
#[cfg(test)]
#[path = "property_scroll_paint_tests.rs"]
mod scroll_paint_tests;
#[cfg(test)]
#[path = "property_scroll_tests.rs"]
mod scroll_tests;
#[cfg(test)]
#[path = "property_sections_tests.rs"]
mod sections_tests;
#[cfg(test)]
#[path = "property_panel_tests.rs"]
mod tests;
