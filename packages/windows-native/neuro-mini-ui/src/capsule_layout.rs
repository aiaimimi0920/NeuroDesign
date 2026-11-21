//! 胶囊与外侧连接点的同源物理几何；展开只改变宽度和锚点，不改变行高。
use crate::{layout::px, Frame};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CapsuleSide {
    Input,
    Output,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CapsuleSize {
    Normal,
    Compact,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CapsuleLayout {
    pub frame: Frame,
    pub anchor: Frame,
    pub body: Frame,
    pub expanded: bool,
    pub side: CapsuleSide,
}
impl CapsuleLayout {
    /// edge 是收起时的锚点中心。输入向左、输出向右展开，图片侧边缘保持不动。
    pub fn new(
        edge: (i32, i32),
        dpi: u32,
        side: CapsuleSide,
        expanded: bool,
        size: CapsuleSize,
    ) -> Self {
        let diameter = px(if size == CapsuleSize::Compact { 8 } else { 28 }, dpi);
        let expanded = expanded && size == CapsuleSize::Normal;
        let width = if expanded { px(256, dpi) } else { diameter };
        let left =
            edge.0
                .saturating_sub(diameter / 2)
                .saturating_sub(if side == CapsuleSide::Input {
                    width - diameter
                } else {
                    0
                });
        let top = edge.1.saturating_sub(diameter / 2);
        let anchor_x = left.saturating_add(if side == CapsuleSide::Output {
            width - diameter
        } else {
            0
        });
        let body_left = left.saturating_add(if side == CapsuleSide::Input {
            diameter
        } else {
            0
        });
        Self {
            frame: Frame {
                x: left,
                y: top,
                width,
                height: diameter,
            },
            anchor: Frame {
                x: anchor_x,
                y: top,
                width: diameter,
                height: diameter,
            },
            body: Frame {
                x: body_left,
                y: top,
                width: width - diameter,
                height: diameter,
            },
            expanded,
            side,
        }
    }
    pub fn connection(self) -> (i32, i32) {
        (
            self.anchor.x.saturating_add(self.anchor.width / 2),
            self.anchor.y.saturating_add(self.anchor.height / 2),
        )
    }
    /// 输入为名称/动作，输出为动作/名称；输出内容从对象侧 18 DIP 开始，不整体镜像。
    pub fn inline_action(self, dpi: u32) -> Option<Frame> {
        self.expanded.then(|| Frame {
            x: match self.side {
                CapsuleSide::Input => self.body.x + px(82, dpi),
                CapsuleSide::Output => self.body.x + px(18, dpi),
            },
            y: self.body.y + px(6, dpi),
            width: px(16, dpi),
            height: px(16, dpi),
        })
    }
    pub fn label(self, dpi: u32, has_action: bool) -> Option<Frame> {
        self.expanded.then(|| {
            let inset = if self.side == CapsuleSide::Output {
                if has_action {
                    44
                } else {
                    18
                }
            } else {
                4
            };
            Frame {
                x: self.body.x + px(inset, dpi),
                width: if has_action {
                    px(72, dpi)
                } else {
                    self.body.width - px(inset + 4, dpi)
                },
                ..self.body
            }
        })
    }
}
