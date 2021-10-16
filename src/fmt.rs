use crate::sys;
use core::{
    fmt::{self, Write},
    mem,
};

const BUF_SIZE: usize = 16;

#[derive(Debug, Clone, Copy)]
pub struct Writer(pub sys::FileDesc);

impl Writer {
    pub fn write_fmt(
        &self,
        arguments: fmt::Arguments,
    ) -> Result<(), sys::Error> {
        let mut adapter = Adapter::new(self.0);
        let _ = adapter.write_fmt(arguments);
        adapter.finish()
    }
}

#[derive(Debug)]
struct Adapter {
    desc: sys::FileDesc,
    buffer: [u8; BUF_SIZE],
    cursor: usize,
    result: Result<(), sys::Error>,
}

impl Adapter {
    fn new(desc: sys::FileDesc) -> Self {
        Self { desc, buffer: [0; BUF_SIZE], cursor: 0, result: Ok(()) }
    }

    fn flush(&mut self) -> Result<(), sys::Error> {
        sys::write(self.desc, &self.buffer[.. self.cursor])?;
        self.buffer = [0; BUF_SIZE];
        self.cursor = 0;
        Ok(())
    }

    fn finish(mut self) -> Result<(), sys::Error> {
        mem::replace(&mut self.result, Ok(()))
    }
}

impl Write for Adapter {
    fn write_str(&mut self, data: &str) -> fmt::Result {
        let mut bytes = data.as_bytes();

        while bytes.len() > 0 && self.result.is_ok() {
            let chunk = BUF_SIZE - self.cursor;
            if bytes.len() > chunk {
                self.buffer[self.cursor ..].copy_from_slice(&bytes[.. chunk]);
                self.cursor = BUF_SIZE;
                self.result = self.flush();
                bytes = &bytes[chunk ..];
            } else {
                let end = self.cursor + data.len();
                self.buffer[self.cursor .. end].copy_from_slice(bytes);
                self.cursor = end;
                bytes = &bytes[bytes.len() ..];
            }
        }

        match self.result {
            Ok(_) => Ok(()),
            Err(_) => Err(fmt::Error),
        }
    }
}

impl Drop for Adapter {
    fn drop(&mut self) {
        let _ = self.flush();
    }
}
