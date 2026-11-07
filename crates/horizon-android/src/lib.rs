//! Horizon on Android. The APK's NativeActivity loads `libhorizon_android.so` and calls
//! `android_main`; this sets up the folders and hands winit the activity, then starts the game
//! as the desktop executable does.
//!
//! Folders, all in the app's external files folder
//! (`/sdcard/Android/data/dev.horizon.android/files`, reachable over USB):
//! - `resources/`: the prepared resources (`horizon-resources.json` and the rest), made on a
//!   computer by the Horizon launcher from the player's own Minecraft: Java Edition.
//! - the game folder itself: `saves/`, `options.txt`, `resourcepacks/`, logs.
#![cfg(target_os = "android")]

use horizon_client_input::platform::{self, AndroidApp};

#[global_allocator]
static ALLOCATOR: horizon_threads::Allocator = horizon_threads::ALLOCATOR;

/// Kept in the library as the desktop executable keeps it (`horizon_mods::api_level::MARK`).
static MOD_API_LEVEL: [u8; 27] = horizon_mods::api_level::MARK;

#[unsafe(no_mangle)]
fn android_main(app: AndroidApp) {
    std::hint::black_box(&MOD_API_LEVEL);
    android_logger::init_once(
        android_logger::Config::default()
            .with_max_level(log::LevelFilter::Info)
            .with_tag("horizon"),
    );
    let dir = app
        .external_data_path()
        .or_else(|| app.internal_data_path())
        .unwrap_or_else(|| "/data/local/tmp/horizon".into());
    if let Err(e) = std::fs::create_dir_all(dir.join("resources")) {
        log::error!("Horizon: game folder {}: {e}", dir.display());
    }
    // SAFETY: nothing else runs yet: the game's threads start after this.
    unsafe {
        std::env::set_var("HORIZON_HOME", &dir);
        std::env::set_var("HORIZON_RESOURCES", dir.join("resources"));
    }
    if let Err(e) = std::env::set_current_dir(&dir) {
        log::error!("Horizon: game folder {}: {e}", dir.display());
    }
    // Panics go to logcat as well as the crash report.
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        log::error!("Horizon panicked: {info}");
        previous(info);
    }));
    log::info!("Horizon for Android, game folder {}", dir.display());
    platform::set_android_app(app);
    horizon_client::entry::main();
}
