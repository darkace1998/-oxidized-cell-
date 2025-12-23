//! Reservation system for SPU atomic operations

use std::sync::atomic::{AtomicU64, Ordering};

/// Reservation for SPU atomic operations (GETLLAR/PUTLLC)
///
/// Each reservation tracks a 128-byte cache line with a version counter
/// and lock bit for atomic conditional stores.
#[repr(C, align(64))]
pub struct Reservation {
    /// Timestamp (version counter) with lock bit in LSB
    timestamp: AtomicU64,
}

impl Reservation {
    /// Lock bit position in timestamp
    pub const LOCK_BIT: u64 = 1;

    /// Create a new reservation
    pub const fn new() -> Self {
        Self {
            timestamp: AtomicU64::new(0),
        }
    }

    /// Acquire the current timestamp (excluding lock bit)
    #[inline]
    pub fn acquire(&self) -> u64 {
        self.timestamp.load(Ordering::Acquire) & !Self::LOCK_BIT
    }

    /// Try to lock the reservation if timestamp matches
    #[inline]
    pub fn try_lock(&self, expected_time: u64) -> bool {
        self.timestamp
            .compare_exchange(
                expected_time,
                expected_time | Self::LOCK_BIT,
                Ordering::AcqRel,
                Ordering::Relaxed,
            )
            .is_ok()
    }

    /// Unlock and increment the timestamp
    #[inline]
    pub fn unlock_and_increment(&self) {
        let current = self.timestamp.load(Ordering::Relaxed);
        let new_time = (current & !Self::LOCK_BIT) + 128; // Increment by cache line size
        self.timestamp.store(new_time, Ordering::Release);
    }

    /// Check if the reservation is locked
    #[inline]
    pub fn is_locked(&self) -> bool {
        (self.timestamp.load(Ordering::Acquire) & Self::LOCK_BIT) != 0
    }

    /// Invalidate the reservation (increment timestamp without locking)
    #[inline]
    pub fn invalidate(&self) {
        self.timestamp.fetch_add(128, Ordering::Release);
    }
}

impl Default for Reservation {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reservation_basic() {
        let res = Reservation::new();
        
        // Initial timestamp should be 0
        assert_eq!(res.acquire(), 0);
        assert!(!res.is_locked());
    }

    #[test]
    fn test_reservation_lock_unlock() {
        let res = Reservation::new();
        
        let time = res.acquire();
        assert!(res.try_lock(time));
        assert!(res.is_locked());
        
        res.unlock_and_increment();
        assert!(!res.is_locked());
        
        let new_time = res.acquire();
        assert_eq!(new_time, 128);
    }

    #[test]
    fn test_reservation_lock_conflict() {
        let res = Reservation::new();
        
        let time = res.acquire();
        assert!(res.try_lock(time));
        
        // Second lock attempt should fail
        assert!(!res.try_lock(time));
        
        res.unlock_and_increment();
    }

    #[test]
    fn test_reservation_invalidate() {
        let res = Reservation::new();
        
        res.invalidate();
        assert_eq!(res.acquire(), 128);
        
        res.invalidate();
        assert_eq!(res.acquire(), 256);
    }
}
