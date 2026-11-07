//! The window system's event loop, where a platform needs more than `EventLoop::new`: on
//! Android winit needs the activity the app was started with (`android_main`'s `AndroidApp`).
use winit::error::EventLoopError;
use winit::event_loop::EventLoop;

#[cfg(target_os = "android")]
pub use winit::platform::android::activity::AndroidApp;

#[cfg(target_os = "android")]
static ANDROID_APP: std::sync::OnceLock<AndroidApp> = std::sync::OnceLock::new();

/// The activity `android_main` received; set once, before the game starts.
#[cfg(target_os = "android")]
pub fn set_android_app(app: AndroidApp) {
    let _ = ANDROID_APP.set(app);
}

/// The activity, for the parts of the game that ask Android for something (the soft keyboard).
#[cfg(target_os = "android")]
pub fn android_app() -> Option<&'static AndroidApp> {
    ANDROID_APP.get()
}

/// The game's event loop.
pub fn event_loop() -> Result<EventLoop<()>, EventLoopError> {
    #[cfg(target_os = "android")]
    {
        use winit::platform::android::EventLoopBuilderExtAndroid;
        let mut builder = EventLoop::builder();
        if let Some(app) = ANDROID_APP.get() {
            builder.with_android_app(app.clone());
        }
        builder.build()
    }
    #[cfg(not(target_os = "android"))]
    EventLoop::new()
}
