fn main() {
  // Check for spaces in path because 'windres' (GNU) fails on them
  if let Ok(cwd) = std::env::current_dir() {
      if cwd.to_string_lossy().contains(' ') {
          println!("cargo:warning=----------------------------------------------------------------");
          println!("cargo:warning=CRITICAL: Project path contains spaces: '{}'", cwd.display());
          println!("cargo:warning=This causes 'windres' to fail during Windows builds.");
          println!("cargo:warning=Please move the project to a path without spaces (e.g. C:\\Projects\\Furukawa).");
          println!("cargo:warning=----------------------------------------------------------------");
      }
  }
  tauri_build::build()
}
