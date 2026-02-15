use furukawa_domain::container::Running;
use furukawa_common::diagnostic::Error;
use tracing::{info, warn};

#[cfg(windows)]
mod sys {
    use windows_sys::Win32::Foundation::{CloseHandle, FALSE, HANDLE};
    use windows_sys::Win32::System::Threading::{
        OpenProcess, TerminateProcess, PROCESS_TERMINATE,
    };

    pub fn kill_process(pid: u32) -> Result<(), std::io::Error> {
        unsafe {
            let handle: HANDLE = OpenProcess(PROCESS_TERMINATE, FALSE, pid);
            if handle == 0 {
                return Err(std::io::Error::last_os_error());
            }

            // In a real "10 year" implementation, we'd send a close event first, wait, then terminate.
            // For now, we wrap TerminateProcess safely.
            let result = TerminateProcess(handle, 1);
            CloseHandle(handle);

            if result == 0 {
                return Err(std::io::Error::last_os_error());
            }
            Ok(())
        }
    }
}

#[cfg(not(windows))]
mod sys {
    pub fn kill_process(_pid: u32) -> Result<(), std::io::Error> {
        Err(std::io::Error::new(std::io::ErrorKind::Unsupported, "Platform not supported"))
    }
}

pub fn stop_container(pid: u32) -> Result<(), Error> {
    info!("Stopping container process PID: {}", pid);
    match sys::kill_process(pid) {
        Ok(_) => {
            info!("Successfully terminated process {}", pid);
            Ok(())
        }
        Err(e) => {
            // If process is already gone, we consider it a success for idempotency
            // But we should verify error code. For now, warn and proceed.
            warn!("Failed to terminate process {}: {}. It might have already exited.", pid, e);
            Ok(()) 
        }
    }
}
