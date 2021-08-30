use std::{convert::TryInto, ffi::CString, os::raw, path::Path, ptr};
use std::borrow::Borrow;
use std::ops::Deref;
use std::time::Duration;

use android_logger::Config;
use log::Level;
use skia_safe::{Budgeted, Canvas, gpu, ImageInfo};
use self::egl::NativeWindowType;
use skia_safe::gpu::gl::Interface;
use skia_safe::gpu::ContextOptions;

use crate::window::ANativeWindow;
use crate::window::window::Window;

extern crate khronos_egl as egl;

pub type SizeT = raw::c_ulong;

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

struct App {
    window: Option<Window>,
}

extern "C" {
    pub fn ANativeWindow_getWidth(window: *mut ANativeWindow) -> i32;
    pub fn ANativeWindow_getHeight(window: *mut ANativeWindow) -> i32;
    pub fn ANativeWindow_release(window: *mut ANativeWindow);
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
    let app = App::get_app(activity);
    if let None = &app.window {
        app.window = Some(Window::from_native_window(window_ptr));
    } else {
        error!("window created without destroyed the last one");
    }
}

pub extern "C" fn on_window_resized(activity: *mut ANativeActivity, window_ptr: *mut ANativeWindow) {
    info!("Native window resized, window = {:?}", window_ptr);
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
    let window = &mut App::get_app(activity).window;
    if let Some(window) = window {
        window.on_paint();
    }
}

pub extern "C" fn on_window_destroyed(activity: *mut ANativeActivity, window_ptr: *mut ANativeWindow) {
    info!("Native window destroyed, window = {:?}", window_ptr);
    let app = App::get_app(activity);
    app.window = None;
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
        callbacks.on_native_window_destroyed = Some(on_window_destroyed);
        // callbacks.onInputQueueCreated = Some(on_input_queue_created);
        // callbacks.onInputQueueDestroyed = Some(on_input_queue_destroyed);

        App::init(activity);
    }
}
