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

      // Spawn the sidecar (furukawad)
      use tauri_plugin_shell::ShellExt;
      let sidecar_command = app.shell().sidecar("furukawad").map_err(|e| e.to_string())?;
      let (mut _rx, _child) = sidecar_command.spawn().map_err(|e| e.to_string())?;
      
      tauri::async_runtime::spawn(async move {
          while let Some(event) = _rx.recv().await {
              if let tauri_plugin_shell::process::CommandEvent::Stdout(line_bytes) = event {
                   let line = String::from_utf8_lossy(&line_bytes);
                   log::info!("furukawad: {}", line);
              }
          }
      });

      Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
