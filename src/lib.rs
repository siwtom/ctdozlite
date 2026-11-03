pub mod audio;
pub mod call;
pub mod chat;
pub mod crypto;
pub mod endpoints;
pub mod gui;
pub mod logger;
pub mod login;
pub mod media;
pub mod param_cipher;
pub mod paths;
pub mod qr;
pub mod session;
pub mod socket;
pub mod store;
pub mod tui;
pub mod ui;
pub mod voip_log;

/// Launch the Zalo Native Multi-Account GUI in Mobile/Embedded mode with a custom sandbox directory.
/// Used by iOS (`start_zalo_ios_app`) and Android (`android_main`) entry points.
pub fn run_mobile_gui(data_dir: Option<std::path::PathBuf>) -> anyhow::Result<()> {
    if let Some(dir) = data_dir {
        paths::set_app_data_dir(dir);
    }
    logger::init(false);

    let mut s = session::Session::new_for_slot(0, false, false, false)?;
    let sess_file = s.session_file_path();
    let _ = s.load_from_file(&sess_file);

    let mut initial_threads = chat::load_threads_cache_for_slot(0);
    store::load_history_into_threads(&s.history_db_path(), &mut initial_threads, 200);

    let viewport = eframe::egui::ViewportBuilder::default()
        .with_decorations(false)
        .with_inner_size([412.0, 820.0])
        .with_min_inner_size([320.0, 480.0])
        .with_title("Zalo");

    let native_options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "Zalo - Rust Mobile Client",
        native_options,
        Box::new(move |cc| {
            let mut app = gui::ZaloGuiApp::new(cc, s, false, initial_threads);
            app.mobile_view_mode = true;
            Ok(Box::new(app))
        }),
    )
    .map_err(|e| anyhow::anyhow!("Không thể khởi chạy Mobile GUI: {e}"))
}

/// C-ABI Entry Point for iOS (`UIApplicationMain` / Swift wrapper).
/// Pass the iOS app's `NSDocumentDirectory` UTF-8 path so all multi-account sessions,
/// SQLite databases, and sticker caches are stored inside the iOS sandbox.
#[no_mangle]
pub extern "C" fn start_zalo_ios_app(sandbox_docs_dir: *const std::os::raw::c_char) {
    let data_dir = if !sandbox_docs_dir.is_null() {
        unsafe { std::ffi::CStr::from_ptr(sandbox_docs_dir) }
            .to_str()
            .ok()
            .map(std::path::PathBuf::from)
    } else {
        None
    };
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed to build Tokio runtime on iOS");
    let _guard = rt.enter();
    let _ = run_mobile_gui(data_dir);
}

/// Entry point for Android NativeActivity (`cargo-apk` / `cdylib`).
/// Automatically configures the Android internal storage sandbox directory (`internal_data_path`)
/// and launches `ZaloGuiApp` in Mobile 1-Column Touch mode with Multi-Account support.
#[cfg(target_os = "android")]
#[no_mangle]
pub fn android_main(app: winit::platform::android::activity::AndroidApp) {
    if let Some(dir) = app.internal_data_path() {
        paths::set_app_data_dir(dir);
    }
    logger::init(false);

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed to build Tokio runtime on Android");
    let _guard = rt.enter();

    let mut s = session::Session::new_for_slot(0, false, false, false)
        .expect("Failed to initialize Session on Android");
    let sess_file = s.session_file_path();
    let _ = s.load_from_file(&sess_file);

    let mut initial_threads = chat::load_threads_cache_for_slot(0);
    store::load_history_into_threads(&s.history_db_path(), &mut initial_threads, 200);

    let viewport = eframe::egui::ViewportBuilder::default()
        .with_decorations(false)
        .with_title("Zalo");

    let mut native_options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };
    native_options.event_loop_builder = Some(Box::new(move |builder| {
        use winit::platform::android::EventLoopBuilderExtAndroid;
        builder.with_android_app(app);
    }));

    let _ = eframe::run_native(
        "Zalo - Rust Android Client",
        native_options,
        Box::new(move |cc| {
            let mut gui_app = gui::ZaloGuiApp::new(cc, s, false, initial_threads);
            gui_app.mobile_view_mode = true;
            Ok(Box::new(gui_app))
        }),
    );
}

