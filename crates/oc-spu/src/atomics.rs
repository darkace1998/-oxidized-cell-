//! SPU atomic operations (GETLLAR/PUTLLC)
//!
//! These implement the SPU's atomic memory operations using
//! the 128-byte reservation system.

/// Atomic operation result
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AtomicResult {
    /// Operation succeeded
    Success,
    /// Reservation was lost
    Lost,
    /// Operation failed
    Failed,
}
