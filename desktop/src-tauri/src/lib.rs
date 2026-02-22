use tauri::Manager;
use tauri_plugin_shell::ShellExt;
use tauri_plugin_shell::process::CommandEvent;

// Keep the child process handle alive for the lifetime of the app.
struct SidecarGuard(tauri_plugin_shell::process::CommandChild);

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .plugin(tauri_plugin_shell::init())
    .setup(|app| {
      if cfg!(debug_assertions) {
        app.handle().plugin(
          tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build(),
        )?;
      }

      // Determine the data directory for furukawad
      let app_data_dir = app.path().app_data_dir()
        .expect("Failed to resolve app data dir");
      
      // Spawn the sidecar (furukawad) with the data directory as env var
      let sidecar_command = app.shell()
        .sidecar("furukawad")
        .map_err(|e| e.to_string())?
        .env("FURUKAWA_DATA_DIR", app_data_dir.to_string_lossy().as_ref());
      
      let (mut rx, child) = sidecar_command.spawn().map_err(|e| e.to_string())?;

      // Log sidecar output in background task
      let handle = app.handle().clone();
      tauri::async_runtime::spawn(async move {
          while let Some(event) = rx.recv().await {
              match event {
                  CommandEvent::Stdout(line_bytes) => {
                      let line = String::from_utf8_lossy(&line_bytes);
                      log::info!("[furukawad stdout] {}", line);
                  }
                  CommandEvent::Stderr(line_bytes) => {
                      let line = String::from_utf8_lossy(&line_bytes);
                      log::error!("[furukawad stderr] {}", line);
                  }
                  CommandEvent::Terminated(payload) => {
                      log::error!("[furukawad] terminated with code: {:?}", payload.code);
                      // Optionally show a dialog here
                      break;
                  }
                  _ => {}
              }
          }
          // Suppress unused variable warning
          let _ = handle;
      });

      // Store guard in managed state so it lives as long as the app
      app.manage(SidecarGuard(child));

      Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
