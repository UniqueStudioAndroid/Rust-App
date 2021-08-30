use khronos_egl as egl;
use skia_safe::gpu::DirectContext as DirectContext;
use skia_safe::Surface as SkSurface;
use crate::window::ANativeWindow;

pub struct GLContext {
    egl: egl::Instance<egl::Static>,
    egl_context: egl::Context,
    egl_display: egl::Display,
    egl_surface: egl::Surface,
    direct_context: DirectContext,
    pub width: i32,
    pub height: i32,
    pub surface: SkSurface,
}

impl Drop for GLContext {
    fn drop(&mut self) {
        info!("drop gl context");
        self.direct_context.abandon();
        self.egl.make_current(self.egl_display, None, None, None).expect("make current for drop failed");
        self.egl.destroy_surface(self.egl_display, self.egl_surface).expect("destroy surface failed");
        self.egl.destroy_context(self.egl_display, self.egl_context).expect("destroy context failed");
    }
}

impl GLContext {
    pub fn new(window_ptr: *mut ANativeWindow, width: i32, height: i32) -> Self {
        let egl = egl::Instance::new(egl::Static);

        // initialize display
        let egl_display = egl.get_display(egl::DEFAULT_DISPLAY)
            .expect("get default display failed");
        assert_ne!(egl_display.as_ptr(), egl::NO_DISPLAY);
        egl.initialize(egl_display).expect("initialize egl failed");

        egl.bind_api(egl::OPENGL_ES_API).expect("unable to bind opengl es api");

        // initialize egl context
        let attributes = [
            egl::RED_SIZE, 8,
            egl::GREEN_SIZE, 8,
            egl::BLUE_SIZE, 8,
            egl::NONE
        ];
        let config = egl.choose_first_config(egl_display, &attributes).expect("unable to find an appropriate ELG configuration").unwrap();
        let context_attributes = [
            egl::CONTEXT_MAJOR_VERSION, 3,
            egl::NONE
        ];
        let egl_context = egl.create_context(egl_display, config, None, &context_attributes).expect("unable to create egl context");
        assert_ne!(egl_context.as_ptr(), egl::NO_CONTEXT);

        // initialize egl surface
        let egl_surface = unsafe {
            egl.create_window_surface(egl_display, config, window_ptr as egl::NativeWindowType, None).expect("create window surface failed")
        };
        assert_ne!(egl_surface.as_ptr(), egl::NO_SURFACE);

        egl.make_current(egl_display, Some(egl_surface), Some(egl_surface), Some(egl_context)).expect("unable to make current");

        let mut direct_context = DirectContext::new_gl(None, None).expect("unable to create direct context");

        // create skia surface
        let fb_info = skia_safe::gpu::gl::FramebufferInfo {
            fboid: 0,
            format: skia_safe::gpu::gl::Format::RGBA8.into(),
        };
        let backend_rt = skia_safe::gpu::BackendRenderTarget::new_gl(
            (width, height),
            None,
            0,
            fb_info
        );
        let surface = skia_safe::Surface::from_backend_render_target(
            &mut direct_context,
            &backend_rt,
            skia_safe::gpu::SurfaceOrigin::TopLeft,
            skia_safe::ColorType::RGBA8888,
            None,
            None
        ).expect("Create skia surface failed");


        GLContext {
            egl,
            egl_display,
            egl_context,
            egl_surface,
            direct_context,
            surface,
            width,
            height,
        }
    }

    pub fn swap_buffer(&self) {
        self.egl.swap_buffers(self.egl_display, self.egl_surface);
    }
}