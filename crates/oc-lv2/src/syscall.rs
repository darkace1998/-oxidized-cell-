//! System call dispatcher

use oc_core::error::KernelError;

/// System call handler
pub struct SyscallHandler;

impl SyscallHandler {
    /// Create a new syscall handler
    pub fn new() -> Self {
        Self
    }

    /// Handle a system call
    pub fn handle(&self, syscall_num: u64, args: &[u64; 8]) -> Result<i64, KernelError> {
        match syscall_num {
            // sys_process_getpid
            1 => Ok(1),
            
            // sys_process_exit
            2 => {
                tracing::info!("sys_process_exit({})", args[0] as i32);
                Ok(0)
            }
            
            // sys_process_get_sdk_version
            25 => Ok(0x00360001), // SDK 3.60
            
            // sys_ppu_thread_yield
            43 => Ok(0),
            
            // sys_ppu_thread_get_id
            44 => Ok(1), // Return thread ID 1
            
            // sys_time_get_system_time
            120 => {
                // Return current time in microseconds
                use std::time::{SystemTime, UNIX_EPOCH};
                let duration = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default();
                Ok(duration.as_micros() as i64)
            }
            
            // sys_time_get_timebase_frequency
            123 => Ok(79800000), // 79.8 MHz timebase
            
            // sys_memory_allocate
            324 => {
                tracing::debug!("sys_memory_allocate(size=0x{:x})", args[0]);
                // Placeholder - would allocate memory
                Ok(0)
            }
            
            // sys_memory_free
            325 => {
                tracing::debug!("sys_memory_free(addr=0x{:x})", args[0]);
                Ok(0)
            }
            
            // sys_tty_write
            403 => {
                // TTY write (console output)
                let _ch = args[0] as u32;
                let _buf = args[1] as u32;
                let _len = args[2] as u32;
                Ok(args[2] as i64)
            }
            
            _ => {
                tracing::warn!("Unknown syscall {}", syscall_num);
                Err(KernelError::UnknownSyscall(syscall_num))
            }
        }
    }
}

impl Default for SyscallHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_syscall_handler() {
        let handler = SyscallHandler::new();
        let args = [0u64; 8];
        
        // Test getpid
        let result = handler.handle(1, &args).unwrap();
        assert_eq!(result, 1);
        
        // Test get_sdk_version
        let result = handler.handle(25, &args).unwrap();
        assert_eq!(result, 0x00360001);
    }
}
