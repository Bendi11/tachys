use tiny_skia::{Color, Paint, PathBuilder, Pixmap, PixmapMut, Rect, Shader, Stroke, Transform};


pub trait Widget {

}

pub trait PixmapExtensions {
    fn outline_rect(&mut self, rect: Rect, color: Color);
}

impl PixmapExtensions for PixmapMut<'_> {
    fn outline_rect(&mut self, rect: Rect, color: Color) {
        let mut path = PathBuilder::new();
        path.move_to(rect.left(), rect.top());
        path.line_to(rect.right(), rect.top());
        path.line_to(rect.right(), rect.bottom());
        path.line_to(rect.left(), rect.bottom());
        path.line_to(rect.left(), rect.top());

        let path = path.finish().unwrap();
        self.stroke_path(
            &path,
            &Paint {
                shader: Shader::SolidColor(color),
                anti_alias: false,
                ..Default::default()
            },
            &Stroke {
                width: 1f32,
                ..Default::default()
            },
            Transform::default(),
            None,
        );
    }
}

impl PixmapExtensions for Pixmap {
    fn outline_rect(&mut self, rect: Rect, color: Color) {
        self.as_mut().outline_rect(rect, color)
    }
}
