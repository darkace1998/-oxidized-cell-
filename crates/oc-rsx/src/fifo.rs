//! RSX command FIFO

use std::collections::VecDeque;

/// RSX command
#[derive(Debug, Clone, Copy)]
pub struct RsxCommand {
    /// Method address
    pub method: u32,
    /// Data value
    pub data: u32,
}

/// Command FIFO for RSX
pub struct CommandFifo {
    /// Command queue
    queue: VecDeque<RsxCommand>,
    /// Get pointer (read position)
    get: u32,
    /// Put pointer (write position)
    put: u32,
    /// Reference value
    reference: u32,
}

impl CommandFifo {
    /// Create a new command FIFO
    pub fn new() -> Self {
        Self {
            queue: VecDeque::with_capacity(4096),
            get: 0,
            put: 0,
            reference: 0,
        }
    }

    /// Push a command to the FIFO
    pub fn push(&mut self, cmd: RsxCommand) {
        self.queue.push_back(cmd);
        self.put += 4;
    }

    /// Pop a command from the FIFO
    pub fn pop(&mut self) -> Option<RsxCommand> {
        let cmd = self.queue.pop_front();
        if cmd.is_some() {
            self.get += 4;
        }
        cmd
    }

    /// Check if FIFO is empty
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    /// Get queue length
    pub fn len(&self) -> usize {
        self.queue.len()
    }

    /// Get current get pointer
    pub fn get_ptr(&self) -> u32 {
        self.get
    }

    /// Get current put pointer
    pub fn put_ptr(&self) -> u32 {
        self.put
    }

    /// Set reference value
    pub fn set_reference(&mut self, value: u32) {
        self.reference = value;
    }

    /// Get reference value
    pub fn get_reference(&self) -> u32 {
        self.reference
    }
}

impl Default for CommandFifo {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fifo_operations() {
        let mut fifo = CommandFifo::new();

        assert!(fifo.is_empty());

        fifo.push(RsxCommand { method: 0x100, data: 0x1234 });
        assert!(!fifo.is_empty());
        assert_eq!(fifo.len(), 1);

        let cmd = fifo.pop().unwrap();
        assert_eq!(cmd.method, 0x100);
        assert_eq!(cmd.data, 0x1234);
        assert!(fifo.is_empty());
    }
}
