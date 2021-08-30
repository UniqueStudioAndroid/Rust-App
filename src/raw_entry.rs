use std::{convert::TryInto, ffi::CString, os::raw, path::Path, ptr};
use std::borrow::Borrow;
use std::ops::Deref;
use std::thread::sleep;
use std::time::Duration;

use android_logger::Config;
use log::Level;
use skia_safe::{Budgeted, Canvas, gpu, ImageInfo};
use self::egl::NativeWindowType;
use skia_safe::gpu::gl::Interface;
use skia_safe::gpu::ContextOptions;

extern crate khronos_egl as egl;

pub type SizeT = raw::c_ulong;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ANativeWindow {
    _unused: [u8; 0],
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct AInputQueue {
    _unused: [u8; 0],
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ARect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

struct Window {
    // gl_context: GLContext<NativeGLContext>,
    render_context: gpu::DirectContext,
    egl: egl::Instance<egl::Static>,
    display: egl::Display,
    surface: egl::Surface,
    width: i32,
    height: i32,
}

impl Window {
    fn draw(&mut self) {
        // let img_info = ImageInfo::new_n32_premul((self.width, self.height), None);
        // let mut sk_surface = skia_safe::Surface::new_render_target(
        //     &mut self.render_context, Budgeted::No, &img_info, None, gpu::SurfaceOrigin::TopLeft, None, false,
        // ).unwrap();
        let fb_info = skia_safe::gpu::gl::FramebufferInfo {
            fboid: 0,
            format: skia_safe::gpu::gl::Format::RGBA8.into(),
        };
        let backend_rt = skia_safe::gpu::BackendRenderTarget::new_gl(
            (self.width, self.height),
            None,
            0,
            fb_info
        );
        let mut sk_surface = skia_safe::Surface::from_backend_render_target(
            &mut self.render_context,
            &backend_rt,
            gpu::SurfaceOrigin::TopLeft,
            skia_safe::ColorType::RGBA8888,
            None,
            None
        ).expect("Create skia surface failed");

        let canvas = sk_surface.canvas();
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
        sk_surface.flush_submit_and_sync_cpu();
        self.swap_buffers();

        info!("Window draw ok...");
    }

    fn swap_buffers(&self) {
        self.egl.swap_buffers(self.display, self.surface);
    }

    fn from_window_ptr(window_ptr: *mut ANativeWindow) -> Self {
        let egl = egl::Instance::new(egl::Static);
        let display = egl.get_display(egl::DEFAULT_DISPLAY).unwrap();
        egl.initialize(display).expect("unable to initialize default display");
        assert_ne!(display.as_ptr(), egl::NO_DISPLAY);

        egl.bind_api(egl::OPENGL_ES_API).expect("unable to bind opengl es api");

        let attributes = [
            egl::RED_SIZE, 8,
            egl::GREEN_SIZE, 8,
            egl::BLUE_SIZE, 8,
            egl::NONE
        ];
        let config = egl.choose_first_config(display, &attributes).expect("unable to find an appropriate ELG configuration").unwrap();

        let context_attributes = [
            egl::CONTEXT_MAJOR_VERSION, 3,
            egl::NONE
        ];
        let context = egl.create_context(display, config, None, &context_attributes).expect("unable to create egl context");
        assert_ne!(context.as_ptr(), egl::NO_CONTEXT);

        unsafe {
            let surface = egl.create_window_surface(display, config, window_ptr as NativeWindowType, None).expect("create window surface failed");
            assert_ne!(surface.as_ptr(), egl::NO_SURFACE);

            egl.make_current(display, Some(surface), Some(surface), Some(context)).expect("unable to make current");

            let width = ANativeWindow_getWidth(window_ptr);
            let height = ANativeWindow_getHeight(window_ptr);

            info!("Native window created ok with width = {:?}, height = {:?}", width, height);

            Window {
                render_context: gpu::DirectContext::new_gl(
                    None,
                    None,
                ).unwrap(),
                egl,
                display,
                surface,
                width,
                height,
            }
        }
    }
}

struct App {
    window: Option<Window>,
}

extern "C" {
    pub fn ANativeWindow_getWidth(window: *mut ANativeWindow) -> i32;
    pub fn ANativeWindow_getHeight(window: *mut ANativeWindow) -> i32;
}

impl App {
    fn init(activity: *mut ANativeActivity) {
        let app = Box::new(App {
            window: None
        });
        unsafe { (*activity).instance = (Box::into_raw(app) as *mut std::os::raw::c_void) };
    }

    fn get_app(activity: *mut ANativeActivity) -> &'static mut Self {
        unsafe {
            let instance = (*activity).instance as *mut App;
            Box::leak(Box::from_raw(instance))
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ANativeActivityCallbacks {
    pub on_start: Option<unsafe extern "C" fn(activity: *mut ANativeActivity)>,
    pub on_resume: Option<unsafe extern "C" fn(activity: *mut ANativeActivity)>,
    pub on_save_instance_state: Option<
        unsafe extern "C" fn(
            activity: *mut ANativeActivity,
            out_size: *mut SizeT,
        ) -> *mut raw::c_void,
    >,
    pub on_pause: Option<unsafe extern "C" fn(activity: *mut ANativeActivity)>,
    pub on_stop: Option<unsafe extern "C" fn(activity: *mut ANativeActivity)>,
    pub on_destroy: Option<unsafe extern "C" fn(activity: *mut ANativeActivity)>,
    pub on_window_focus_changed: Option<
        unsafe extern "C" fn(activity: *mut ANativeActivity, has_focus: raw::c_int),
    >,
    pub on_native_window_created: Option<
        unsafe extern "C" fn(activity: *mut ANativeActivity, window: *mut ANativeWindow),
    >,
    pub on_native_window_resized: Option<
        unsafe extern "C" fn(activity: *mut ANativeActivity, window: *mut ANativeWindow),
    >,
    pub on_native_window_redraw_needed: Option<
        unsafe extern "C" fn(activity: *mut ANativeActivity, window: *mut ANativeWindow),
    >,
    pub on_native_window_destroyed: Option<
        unsafe extern "C" fn(activity: *mut ANativeActivity, window: *mut ANativeWindow),
    >,
    pub on_input_queue_created: Option<
        unsafe extern "C" fn(activity: *mut ANativeActivity, queue: *mut AInputQueue),
    >,
    pub on_input_queue_destroyed: Option<
        unsafe extern "C" fn(activity: *mut ANativeActivity, queue: *mut AInputQueue),
    >,
    pub on_content_rect_changed: Option<
        unsafe extern "C" fn(activity: *mut ANativeActivity, rect: *const ARect),
    >,
    pub on_configuration_changed:
    Option<unsafe extern "C" fn(activity: *mut ANativeActivity)>,
    pub on_low_memory: Option<unsafe extern "C" fn(activity: *mut ANativeActivity)>,
}


#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ANativeActivity {
    pub callbacks: *mut ANativeActivityCallbacks,
    pub vm: *mut raw::c_void,
    //JavaVM,
    pub env: *mut raw::c_void,
    //JNIEnv,
    pub clazz: *mut raw::c_void,
    //jobject,
    pub internal_data_path: *const raw::c_char,
    pub external_data_path: *const raw::c_char,
    pub sdk_version: i32,
    pub instance: *mut raw::c_void,
    // pub asset_manager: *mut AAssetManager,
    // pub obb_path: *const raw::c_char,
}

pub extern "C" fn on_window_created(activity: *mut ANativeActivity, window_ptr: *mut ANativeWindow) {
    info!("Native window created, window = {:?}", window_ptr);
    // let app = App::get_app(activity);
    // match app.window {
    //     None => {}
    //     _ => {}
    // }
}

pub extern "C" fn on_window_resized(activity: *mut ANativeActivity, window_ptr: *mut ANativeWindow) {
    unsafe {
        let width = ANativeWindow_getWidth(window_ptr);
        let height = ANativeWindow_getHeight(window_ptr);

        info!("Native window resized, window = {:?}, width = {:?}, height = {:?}", window_ptr, width, height);
    }
    let app = App::get_app(activity);
    if let None = &app.window {
        app.window = Some(Window::from_window_ptr(window_ptr))
    }
    // let app = App::get_app(activity);
    // match &mut app.window {
    //     Some(window) => {
    //         unsafe {
    //             window.width = ANativeWindow_getWidth(window_ptr);
    //             window.height = ANativeWindow_getHeight(window_ptr);
    //         }
    //         info!("width {:} height {:}", window.width, window.height);
    //     }
    //     _ => {}
    // }
}

pub extern "C" fn on_window_redraw(activity: *mut ANativeActivity, window_ptr: *mut ANativeWindow) {
    info!("Native window redraw, window = {:?}", window_ptr);
    let app = App::get_app(activity);
    app.window.as_mut().unwrap().draw();
}

#[no_mangle]
pub extern "C" fn ANativeActivity_onCreate(
    activity: *mut ANativeActivity,
    saved_state: *mut raw::c_void,
    saved_state_size: SizeT,
) {
    android_logger::init_once(
        Config::default()
            .with_min_level(Level::Trace)
            .with_tag("RustApp")
    );

    info!("Native activity instance = {:?}, savedState = {:?}, size = {}", activity, saved_state, saved_state_size);


    unsafe {
        // initialize all callbacks
        let mut callbacks = (*activity).callbacks.as_mut().unwrap();
        // callbacks.on_start = Some(on_start);
        // callbacks.onResume = Some(on_resume);
        // callbacks.on_pause = Some(on_pause);
        // callbacks.on_stop = Some(on_stop);
        // callbacks.onDestroy = Some(on_destroy);
        // callbacks.onWindowFocusChanged = Some(on_window_focus_changed);
        callbacks.on_native_window_created = Some(on_window_created);
        callbacks.on_native_window_resized = Some(on_window_resized);
        callbacks.on_native_window_redraw_needed = Some(on_window_redraw);
        // callbacks.onInputQueueCreated = Some(on_input_queue_created);
        // callbacks.onInputQueueDestroyed = Some(on_input_queue_destroyed);

        App::init(activity);
    }
}
