use self::super::{ANativeWindow, gl_context::GLContext};
use crate::raw_entry::{ANativeWindow_getWidth, ANativeWindow_getHeight};

pub struct Window {
    context: GLContext,
}

impl Window {
    fn get_width_and_height(window_ptr: *mut ANativeWindow) -> (i32, i32) {
        unsafe {
            (ANativeWindow_getWidth(window_ptr), ANativeWindow_getHeight(window_ptr))
        }
    }

    pub fn from_native_window(window_ptr: *mut ANativeWindow) -> Self {
        let (width, height) = Window::get_width_and_height(window_ptr);
        Window {
            context: GLContext::new(window_ptr, width, height)
        }
    }

    pub fn on_paint(&mut self) {
        let mut surface = &mut self.context.surface;

        let canvas = surface.canvas();
        use skia_safe::{Rect, Color4f, ColorSpace, Paint};
        let rect = Rect {
            left: 300.0,
            right: 700.0,
            top: 500.0,
            bottom: 1000.0,
        };
        let color = Color4f {
            r: 1.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        };
        let color_space = ColorSpace::new_srgb();
        let paint = Paint::new(&color, Some(&color_space));
        canvas.draw_rect(rect, &paint);

        surface.flush_and_submit();
        self.context.swap_buffer();
        info!("window draw ok...");
    }
}