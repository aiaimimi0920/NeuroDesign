//! 将调用方提供的静态 SVG 路径转换成 D2D 几何；不读取文件、网络或应用数据。
use svgtypes::{PathParser, PathSegment};
use windows::{
    core::{Error, Result},
    Win32::{
        Foundation::E_INVALIDARG,
        Graphics::Direct2D::{Common::*, *},
    },
};

pub(crate) unsafe fn geometry(
    factory: &ID2D1Factory,
    data: &str,
    scale: f32,
    offset: f32,
    filled: bool,
) -> Result<ID2D1PathGeometry> {
    geometry_at(factory, data, scale, (offset, offset), filled)
}
pub(crate) unsafe fn geometry_at(
    factory: &ID2D1Factory,
    data: &str,
    scale: f32,
    offset: (f32, f32),
    filled: bool,
) -> Result<ID2D1PathGeometry> {
    geometry_rotated(factory, data, scale, offset, filled, (0.0, 0.0, 0.0))
}
pub(crate) unsafe fn geometry_rotated(
    factory: &ID2D1Factory,
    data: &str,
    scale: f32,
    offset: (f32, f32),
    filled: bool,
    rotation: (f32, f32, f32),
) -> Result<ID2D1PathGeometry> {
    let path = factory.CreatePathGeometry()?;
    let sink = path.Open()?;
    let (mut x, mut y, mut start_x, mut start_y) = (0.0, 0.0, 0.0, 0.0);
    let mut open = false;
    let (sin, cos) = rotation.0.to_radians().sin_cos();
    let point = |x: f32, y: f32| D2D_POINT_2F {
        x: offset.0 + (rotation.1 + (x - rotation.1) * cos - (y - rotation.2) * sin) * scale,
        y: offset.1 + (rotation.2 + (x - rotation.1) * sin + (y - rotation.2) * cos) * scale,
    };
    for segment in PathParser::from(data) {
        let segment = segment.map_err(|_| Error::new(E_INVALIDARG, "Invalid toolbar icon path"))?;
        match segment {
            PathSegment::MoveTo { abs, x: a, y: b } => {
                if open {
                    sink.EndFigure(D2D1_FIGURE_END_OPEN);
                }
                x = a as f32 + if abs { 0.0 } else { x };
                y = b as f32 + if abs { 0.0 } else { y };
                start_x = x;
                start_y = y;
                sink.BeginFigure(
                    point(x, y),
                    if filled {
                        D2D1_FIGURE_BEGIN_FILLED
                    } else {
                        D2D1_FIGURE_BEGIN_HOLLOW
                    },
                );
                open = true;
            }
            PathSegment::LineTo { abs, x: a, y: b } => {
                x = a as f32 + if abs { 0.0 } else { x };
                y = b as f32 + if abs { 0.0 } else { y };
                sink.AddLine(point(x, y));
            }
            PathSegment::HorizontalLineTo { abs, x: a } => {
                x = a as f32 + if abs { 0.0 } else { x };
                sink.AddLine(point(x, y));
            }
            PathSegment::VerticalLineTo { abs, y: b } => {
                y = b as f32 + if abs { 0.0 } else { y };
                sink.AddLine(point(x, y));
            }
            PathSegment::CurveTo {
                abs,
                x1,
                y1,
                x2,
                y2,
                x: a,
                y: b,
            } => {
                let (ox, oy) = if abs { (0.0, 0.0) } else { (x, y) };
                sink.AddBezier(&D2D1_BEZIER_SEGMENT {
                    point1: point(x1 as f32 + ox, y1 as f32 + oy),
                    point2: point(x2 as f32 + ox, y2 as f32 + oy),
                    point3: point(a as f32 + ox, b as f32 + oy),
                });
                x = a as f32 + ox;
                y = b as f32 + oy;
            }
            PathSegment::EllipticalArc {
                abs,
                rx,
                ry,
                x_axis_rotation,
                large_arc,
                sweep,
                x: a,
                y: b,
            } => {
                x = a as f32 + if abs { 0.0 } else { x };
                y = b as f32 + if abs { 0.0 } else { y };
                sink.AddArc(&D2D1_ARC_SEGMENT {
                    point: point(x, y),
                    size: D2D_SIZE_F {
                        width: rx as f32 * scale,
                        height: ry as f32 * scale,
                    },
                    rotationAngle: x_axis_rotation as f32 + rotation.0,
                    sweepDirection: if sweep {
                        D2D1_SWEEP_DIRECTION_CLOCKWISE
                    } else {
                        D2D1_SWEEP_DIRECTION_COUNTER_CLOCKWISE
                    },
                    arcSize: if large_arc {
                        D2D1_ARC_SIZE_LARGE
                    } else {
                        D2D1_ARC_SIZE_SMALL
                    },
                });
            }
            PathSegment::ClosePath { .. } => {
                sink.EndFigure(D2D1_FIGURE_END_CLOSED);
                open = false;
                x = start_x;
                y = start_y;
            }
            _ => {
                return Err(Error::new(
                    E_INVALIDARG,
                    "Unsupported toolbar icon path segment",
                ))
            }
        }
    }
    if open {
        sink.EndFigure(D2D1_FIGURE_END_OPEN);
    }
    sink.Close()?;
    Ok(path)
}
