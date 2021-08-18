use ndk_sys::{ANativeWindow, ANativeWindow_getHeight, ANativeWindow_getWidth, 
    ANativeWindow_Buffer, ARect, ANativeWindow_lock, ANativeWindow_unlockAndPost};
use skia_safe::{Canvas, ColorType, AlphaType, ColorSpace, Rect, Color4f, Paint, ImageInfo, ISize};

pub struct Window {
    pub width: i32,
    pub height: i32,
    ptr: *mut ANativeWindow,
}

impl Window {

    pub fn create(ptr: *mut ANativeWindow) -> Window {
        info!("Create window at {:?}", ptr);
        let w: i32;
        let h: i32;
        unsafe {
            w = ANativeWindow_getWidth(ptr);
            h = ANativeWindow_getHeight(ptr);
        }
        Window {
            width: w,
            height: h,
            ptr: ptr
        }
    }
    
    pub fn same(&self, window: *mut ANativeWindow) -> bool {
        self.ptr == window
    }

    pub fn resize(&mut self) {
        unsafe {
            self.width = ANativeWindow_getWidth(self.ptr);
            self.height = ANativeWindow_getHeight(self.ptr);
        }
    }

    pub fn on_paint(&mut self) {
        let mut rect = ARect {
            left: 0 ,
            right: 0,
            top: 0,
            bottom: 0,
        };
        let mut buffer = ANativeWindow_Buffer {
            width: 0,
            height: 0,
            stride: 0,
            format: 0,
            bits: 0 as *mut std::os::raw::c_void,
            reserved: [0; 6],
        };
        let info = ImageInfo::new(
            ISize::new(self.width, self.height),
            ColorType::RGBA8888,
            AlphaType::Opaque,
            ColorSpace::new_srgb(),
        );
        let data: &mut [u8];
        unsafe {
            ANativeWindow_lock(self.ptr, (&mut buffer) as *mut ANativeWindow_Buffer, (&mut rect) as *mut ARect);
            data = std::slice::from_raw_parts_mut(buffer.bits as *mut u8, (self.width * self.height * 32) as usize);
        }

        let mut canvas = Canvas::from_raster_direct(&info, data, Some((buffer.stride * 4) as usize), None).unwrap();
        
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

        self.swap_buffers();
    }

    fn swap_buffers(&self) {
        unsafe {
            ANativeWindow_unlockAndPost(self.ptr);
        }
    }
}
