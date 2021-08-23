// use std::os::raw;
//
// use log::Level;
// use android_logger::Config;
// use ndk_sys::{ANativeActivity, ANativeWindow, AInputQueue};
// use crate::android::App;
// use crate::window::Window;
// use skia_safe::{ImageInfo, ColorType, ColorSpace, AlphaType, ISize, Canvas, Paint, Rect, Color4f};

// #[allow(non_snake_case)]
// #[no_mangle]
// pub fn ANativeActivity_onCreate(
//     activity: *mut ANativeActivity,
//     saved_state: *mut raw::c_void,
//     saved_state_size: raw::c_ulong,
// ) {
//     android_logger::init_once(
//         Config::default()
//         .with_min_level(Level::Trace)
//         .with_tag("RustApp")
//     );
//
//     info!("Native activity instance = {:?}, savedState = {:?}, size = {}", activity, saved_state, saved_state_size);
//
//     unsafe {
//         // initialize all callbacks
//         let mut callbacks = (*activity).callbacks.as_mut().unwrap();
//         callbacks.onStart = Some(on_start);
//         callbacks.onResume = Some(on_resume);
//         callbacks.on_pause = Some(on_pause);
//         callbacks.on_stop = Some(on_stop);
//         callbacks.onDestroy = Some(on_destroy);
//         callbacks.onWindowFocusChanged = Some(on_window_focus_changed);
//         callbacks.onNativeWindowCreated = Some(on_window_created);
//         callbacks.onNativeWindowDestroyed = Some(on_window_destroyed);
//         callbacks.on_native_window_resized = Some(on_window_resized);
//         callbacks.onNativeWindowRedrawNeeded = Some(on_window_redraw);
//         callbacks.onInputQueueCreated = Some(on_input_queue_created);
//         callbacks.onInputQueueDestroyed = Some(on_input_queue_destroyed);
//
//         App::init(activity);
//     }
// }
//
// pub extern "C" fn on_start(_activity: *mut ANativeActivity) {
//     info!("Native activity start");
// }
//
// pub extern "C" fn on_resume(_activity: *mut ANativeActivity) {
//     info!("Native activity resume");
// }
//
// pub extern "C" fn on_pause(_activity: *mut ANativeActivity) {
//     info!("Native activity pause");
// }
//
// pub extern "C" fn on_stop(_activity: *mut ANativeActivity) {
//     info!("Native activity stop");
// }
//
// pub extern "C" fn on_destroy(_activity: *mut ANativeActivity) {
//     info!("Native activity destroy");
// }
//
// pub extern "C" fn on_window_focus_changed(_activity: *mut ANativeActivity, focus: raw::c_int) {
//     info!("Native activity window focus changed, focus = {:?}", focus);
// }
//
// pub extern "C" fn on_window_created(activity: *mut ANativeActivity, window_ptr: *mut ANativeWindow) {
//     info!("Native window created, window = {:?}", window_ptr);
//     let app = App::from_ptr(activity);
//     match &app.window {
//         Some(window) if window.same(window_ptr) => return,
//         _ => app.window = Some(Window::create(window_ptr))
//     };
// }
//
// pub extern "C" fn on_window_resized(activity: *mut ANativeActivity, window_ptr: *mut ANativeWindow) {
//     info!("Native window resized, window = {:?}", window_ptr);
//     App::from_ptr(activity).window.as_mut().unwrap().resize();
// }
//
// pub extern "C" fn on_window_redraw(activity: *mut ANativeActivity, window_ptr: *mut ANativeWindow) {
//     info!("Native window redraw, window = {:?}", window_ptr);
//
//     let app = App::from_ptr(activity);
//     app.window.as_mut().unwrap().on_paint();
// }
//
// pub extern "C" fn on_window_destroyed(_activity: *mut ANativeActivity, window_ptr: *mut ANativeWindow) {
//     info!("Native window created, window = {:?}", window_ptr);
// }
//
// pub extern "C" fn on_input_queue_created(activity: *mut ANativeActivity, queue: *mut AInputQueue) {
//     let app = App::from_ptr(activity);
//     app.looper.bind_queue(queue);
//     info!("Native input queue created, queue = {:?}", queue);
// }
//
// pub extern "C" fn on_input_queue_destroyed(activity: *mut ANativeActivity, queue: *mut AInputQueue) {
//     let app = App::from_ptr(activity);
//     app.looper.unbind_queue(queue);
//     info!("Native input queue destroyed, queue = {:?}", queue);
// }
