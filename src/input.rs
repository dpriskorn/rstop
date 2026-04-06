use crate::logger::Logger;

pub struct InputHandler {
    buf: [u8; 16],
    nonblock_set: bool,
}

impl InputHandler {
    pub fn new() -> Self {
        InputHandler {
            buf: [0; 16],
            nonblock_set: false,
        }
    }

    pub fn read_key(&mut self, logger: &Logger) -> Option<u8> {
        if !self.nonblock_set {
            let flags = unsafe { libc::fcntl(0, libc::F_GETFL) };
            unsafe { libc::fcntl(0, libc::F_SETFL, flags | libc::O_NONBLOCK) };
            self.nonblock_set = true;
        }
        let res = unsafe { libc::read(0, self.buf.as_mut_ptr() as *mut libc::c_void, 1) };
        if res == 1 {
            let key = self.buf[0];
            logger.debug(&format!("read_key: raw byte = {:#04x}", key));
            if key == 0x1b {
                let res2 =
                    unsafe { libc::read(0, self.buf.as_mut_ptr().add(1) as *mut libc::c_void, 1) };
                logger.debug(&format!("read_key: esc seq res2 = {}", res2));
                if res2 == 1 && self.buf[1] == 0x5b {
                    let res3 = unsafe {
                        libc::read(0, self.buf.as_mut_ptr().add(2) as *mut libc::c_void, 1)
                    };
                    logger.debug(&format!("read_key: esc seq res3 = {}", res3));
                    if res3 == 1 {
                        let arrow = self.buf[2];
                        logger.debug(&format!("read_key: arrow = {:#04x}", arrow));
                        match arrow {
                            b'A' => return Some(0xF0),
                            b'B' => return Some(0xF1),
                            b'C' => return Some(0xF2),
                            b'D' => return Some(0xF3),
                            _ => return Some(0x1b),
                        }
                    }
                }
            }
            Some(key)
        } else {
            None
        }
    }

    #[allow(dead_code)]
    pub fn enable_raw_mode(termios: &libc::termios) {
        unsafe { libc::tcsetattr(0, libc::TCSANOW, termios) };
    }

    #[allow(dead_code)]
    pub fn disable_raw_mode(termios: &libc::termios) {
        unsafe { libc::tcsetattr(0, libc::TCSANOW, termios) };
    }
}

impl Default for InputHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_handler_creation() {
        let handler = InputHandler::new();
        assert_eq!(handler.buf[0], 0);
    }
}
