//! 下附着抽屉的物理布局；展开不改变按钮身份，收起只留下居中的三角命中区。
use crate::{layout::px, Frame};
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DrawerLayout {
    pub width: i32,
    pub height: i32,
    pub title: Frame,
    pub buttons: Vec<Frame>,
    pub caret: Frame,
}
impl DrawerLayout {
    /// 裁剪收起标记的底色；顶层宿主也使用同一形状，不复制主题几何。
    /// # Safety
    /// hwnd 属于调用线程；成功后 HRGN 所有权交给系统。
    pub unsafe fn apply_region(
        &self,
        hwnd: windows::Win32::Foundation::HWND,
        expanded: bool,
        dpi: u32,
    ) -> windows::core::Result<()> {
        use windows::Win32::{Foundation::POINT, Graphics::Gdi::*};
        let region = if expanded {
            CreateRectRgn(0, 0, self.width, self.height)
        } else {
            CreatePolygonRgn(
                &[
                    POINT {
                        x: self.caret.x + px(3, dpi),
                        y: px(1, dpi),
                    },
                    POINT {
                        x: self.caret.x + px(17, dpi),
                        y: px(1, dpi),
                    },
                    POINT {
                        x: self.caret.x + px(10, dpi),
                        y: px(9, dpi),
                    },
                ],
                WINDING,
            )
        };
        if region.is_invalid() {
            return Err(windows::core::Error::from_win32());
        }
        if SetWindowRgn(hwnd, Some(region), false) == 0 {
            let error = windows::core::Error::from_win32();
            let _ = DeleteObject(windows::Win32::Graphics::Gdi::HGDIOBJ(region.0));
            return Err(error);
        }
        Ok(())
    }
    pub fn new(title_width: i32, text_buttons: &[bool], expanded: bool, dpi: u32) -> Self {
        let mut x = px(5, dpi) + title_width.clamp(px(12, dpi), px(100, dpi));
        let title = Frame {
            x: px(5, dpi),
            y: px(5, dpi),
            width: x - px(5, dpi),
            height: px(28, dpi),
        };
        let buttons = text_buttons
            .iter()
            .map(|text| {
                x += px(4, dpi);
                let frame = Frame {
                    x,
                    y: px(5, dpi),
                    width: px(if *text { 62 } else { 28 }, dpi),
                    height: px(28, dpi),
                };
                x += frame.width;
                frame
            })
            .collect();
        // 收起也保留同一水平坐标系，避免宿主重新居中时把旧三角帧先搬向左侧。
        // 实际绘制/命中仍由三角 HRGN 限定，不把透明两侧变成输入区。
        let width = x + px(5, dpi);
        let height = px(if expanded { 52 } else { 10 }, dpi);
        Self {
            width,
            height,
            title,
            buttons,
            caret: Frame {
                x: (width - px(20, dpi)) / 2,
                y: if expanded { height - px(14, dpi) } else { 0 },
                width: px(20, dpi),
                height: px(10, dpi),
            },
        }
    }
}
