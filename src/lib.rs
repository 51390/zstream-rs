extern crate libc;

use libz_sys::{
    z_stream,
    z_streamp,
    inflateInit2_,
    inflate,
    inflateEnd,
    uInt,
    voidpf,
    zlibVersion,
};

use log::{info, warn, error};
use std::io::prelude::*;
use std::io::Result;
use std::fs::File;
use std::ptr::null_mut;

unsafe extern "C" fn zalloc(_: voidpf, n: uInt, c: uInt) -> voidpf {
    libc::calloc(n as usize, c as usize)
}

unsafe extern "C" fn zfree(_: voidpf, p: voidpf) {
    libc::free(p)
}

struct Decoder {
    input: Box<dyn Read>,
    stream: z_stream,
    initialized: bool,
}

impl Decoder {
    fn new(input: impl Read + 'static) -> Decoder {
        Decoder {
            initialized: false,
            input: Box::new(input),
            stream: z_stream {
                next_in: null_mut(),
                avail_in: 0,
                total_in: 0,
                next_out: null_mut(),
                avail_out: 0,
                total_out: 0,
                msg: null_mut(),
                state: null_mut(),
                zalloc: zalloc,
                zfree: zfree,
                opaque: null_mut(),
                data_type: 0,
                adler: 0,
                reserved: 0,
            },
        }
    }

    fn stream(&mut self) -> &mut z_stream {
        &mut self.stream
    }
}

impl Read for Decoder {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let mut inner_buf : [u8; 128] = [0; 128];
        let bytes = match self.input.read(&mut inner_buf) {
            Ok(bytes) => bytes,
            Err(e) =>  { return Err(e); },
        };

        if !self.initialized {
            self.stream.next_in = inner_buf.as_mut_ptr();
            self.stream.avail_in = bytes as u32;
            self.stream.next_out = buf.as_mut_ptr();
            self.stream.avail_out = buf.len() as u32;
            unsafe {
                inflateInit2_(&mut self.stream as z_streamp, 16+15, zlibVersion(), 0);
            }
            self.initialized = true;
        }

        Ok(0)
    }
}


#[cfg(test)]
mod tests {
    #[test]
    fn test() {
    }
}
