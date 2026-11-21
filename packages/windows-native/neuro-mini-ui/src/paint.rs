//! 固定主题的 D2D 软件绘制；可失败的资源准备全部在 BeginDraw 之前完成。
use windows::{
    core::Result,
    Win32::{
        Foundation::*,
        Graphics::{
            Direct2D::{Common::*, *},
            Dxgi::Common::DXGI_FORMAT_B8G8R8A8_UNORM,
            Gdi::*,
        },
    },
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Icon {
    pub(crate) source: &'static str,
    pub(crate) svg: bool,
}
impl Icon {
    pub const fn stroke(path: &'static str) -> Self {
        Self {
            source: path,
            svg: false,
        }
    }
    /// 原样消费静态 SVG 的图形与逐元素样式；不加载文件、字体、脚本或外部资源。
    pub const fn svg(source: &'static str) -> Self {
        Self { source, svg: true }
    }
    /// 检查静态图标是否能由原生绘图器完整准备；不创建 HWND 或读取外部内容。
    pub fn validate(self) -> Result<()> {
        unsafe {
            let painter = Painter::new()?;
            crate::icon::prepare(&painter, self, 1.0, (0.0, 0.0))?;
        }
        Ok(())
    }
}
#[derive(Clone, Copy)]
pub struct Palette {
    pub surface: u32,
    pub control: u32,
    pub text: u32,
    pub muted: u32,
    pub signal: u32,
}
impl Default for Palette {
    fn default() -> Self {
        Self {
            surface: 0x090c11,
            control: 0x111720,
            text: 0xf7f8ef,
            muted: 0x929a9f,
            signal: 0xd9ff38,
        }
    }
}
pub(crate) fn color(value: u32, opacity: f32) -> D2D1_COLOR_F {
    D2D1_COLOR_F {
        r: ((value >> 16) & 255) as f32 / 255.0,
        g: ((value >> 8) & 255) as f32 / 255.0,
        b: (value & 255) as f32 / 255.0,
        a: opacity,
    }
}
pub(crate) fn mix(a: u32, b: u32, amount: f32) -> u32 {
    [16, 8, 0].into_iter().fold(0, |result, shift| {
        result
            | (((((a >> shift) & 255) as f32 * (1.0 - amount)
                + ((b >> shift) & 255) as f32 * amount)
                .round() as u32)
                << shift)
    })
}
#[derive(Clone, Copy)]
pub(crate) struct ButtonState {
    pub active: bool,
    pub enabled: bool,
    pub focus: bool,
    pub hot: bool,
    pub pressed: bool,
    pub corner: bool,
    pub dropdown: bool,
    pub size: i32,
}
pub(crate) struct Painter {
    pub(crate) target: ID2D1DCRenderTarget,
    pub(crate) factory: ID2D1Factory,
    pub(crate) palette: Palette,
}
impl Painter {
    pub(crate) unsafe fn new() -> Result<Self> {
        let factory: ID2D1Factory = D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, None)?;
        let target = factory.CreateDCRenderTarget(&D2D1_RENDER_TARGET_PROPERTIES {
            r#type: D2D1_RENDER_TARGET_TYPE_SOFTWARE,
            pixelFormat: D2D1_PIXEL_FORMAT {
                format: DXGI_FORMAT_B8G8R8A8_UNORM,
                alphaMode: D2D1_ALPHA_MODE_IGNORE,
            },
            dpiX: 96.0,
            dpiY: 96.0,
            ..Default::default()
        })?;
        Ok(Self {
            target,
            factory,
            palette: Palette::default(),
        })
    }
    pub(crate) unsafe fn begin(&self, dc: HDC, rect: RECT, background: u32) -> Result<()> {
        self.target.BindDC(dc, &rect)?;
        self.target.BeginDraw();
        self.target.Clear(Some(&color(background, 1.0)));
        Ok(())
    }
    pub(crate) unsafe fn shell(
        &self,
        dc: HDC,
        width: i32,
        height: i32,
        dpi: u32,
        dividers: &[super::Frame],
        join: crate::strip_shell::Join,
    ) -> Result<()> {
        let s = dpi.max(96) as f32 / 96.0;
        let brush = self
            .target
            .CreateSolidColorBrush(&color(mix(self.palette.surface, 0xffffff, 0.12), 1.0), None)?;
        let mut triangles = Vec::new();
        let outline = crate::strip_shell::outline(&self.factory, width, height, dpi, join)?;
        for frame in dividers {
            for (path, y) in [
                ("M0 0h6L3 3Z", frame.y as f32 - s),
                ("M3 0 6 3H0Z", frame.y as f32 + 26.0 * s),
            ] {
                triangles.push(super::path::geometry_at(
                    &self.factory,
                    path,
                    s,
                    (frame.x as f32 + s, y),
                    true,
                )?);
            }
        }
        self.begin(
            dc,
            RECT {
                left: 0,
                top: 0,
                right: width,
                bottom: height,
            },
            self.palette.surface,
        )?;
        self.target.DrawGeometry(&outline, &brush, s, None);
        for frame in dividers {
            let x = frame.x as f32 + frame.width as f32 / 2.0;
            self.target.DrawLine(
                D2D_POINT_2F {
                    x,
                    y: frame.y as f32 + 8.0 * s,
                },
                D2D_POINT_2F {
                    x,
                    y: frame.y as f32 + 20.0 * s,
                },
                &brush,
                s,
                None,
            );
        }
        for triangle in triangles {
            self.target.FillGeometry(&triangle, &brush, None);
        }
        self.target.EndDraw(None, None)
    }
    pub(crate) unsafe fn button(
        &self,
        dc: HDC,
        area: RECT,
        icon: Icon,
        state: ButtonState,
    ) -> Result<()> {
        let ButtonState {
            active,
            enabled,
            focus,
            hot,
            pressed,
            corner,
            dropdown,
            size,
        } = state;
        let width = size as f32;
        let s = width / 28.0;
        // 小三角是真实按钮，但重画完整主按钮的裁切部分，避免覆盖选中描边的暗色方块。
        let origin = if corner {
            (area.right - area.left) as f32 - width
        } else {
            0.0
        };
        let base = mix(self.palette.control, self.palette.text, 0.06);
        let background = if active {
            mix(base, self.palette.signal, 0.12)
        } else if (hot || pressed) && enabled {
            mix(base, self.palette.text, 0.06)
        } else {
            base
        };
        let brush = self
            .target
            .CreateSolidColorBrush(&color(background, 1.0), None)?;
        let factor = 22.5 * s / 24.0;
        let shape = super::icon::prepare(
            self,
            icon,
            factor,
            (
                origin + (width - 22.5 * s) / 2.0,
                origin + (width - 22.5 * s) / 2.0,
            ),
        )?;
        let caret = if dropdown {
            Some(super::path::geometry(
                &self.factory,
                "M2 4.5 6 8l4-3.5H2Z",
                s,
                origin + width - 12.0 * s,
                true,
            )?)
        } else {
            None
        };
        self.begin(dc, area, self.palette.surface)?;
        let rect = D2D1_ROUNDED_RECT {
            rect: D2D_RECT_F {
                left: origin + 2.0 * s,
                top: origin + 2.0 * s,
                right: origin + width - 2.0 * s,
                bottom: origin + width - 2.0 * s,
            },
            radiusX: 5.0 * s,
            radiusY: 5.0 * s,
        };
        self.target.FillRoundedRectangle(&rect, &brush);
        if active || focus {
            brush.SetColor(&color(self.palette.signal, 1.0));
            self.target.DrawRoundedRectangle(
                &D2D1_ROUNDED_RECT {
                    rect: D2D_RECT_F {
                        left: origin + 2.5 * s,
                        top: origin + 2.5 * s,
                        right: origin + width - 2.5 * s,
                        bottom: origin + width - 2.5 * s,
                    },
                    radiusX: 4.5 * s,
                    radiusY: 4.5 * s,
                },
                &brush,
                if focus { 2.0 * s } else { s },
                None,
            );
        }
        let ink = if !enabled {
            self.palette.muted
        } else if active {
            self.palette.signal
        } else if hot {
            self.palette.text
        } else {
            mix(self.palette.surface, self.palette.text, 0.78)
        };
        brush.SetColor(&color(ink, if enabled { 1.0 } else { 0.4 }));
        shape.draw(self, &brush, ink, if enabled { 1.0 } else { 0.4 });
        if let Some(caret) = caret {
            self.target.FillGeometry(&caret, &brush, None);
        }
        self.target.EndDraw(None, None)
    }
}
