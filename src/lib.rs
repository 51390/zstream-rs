extern crate libc;

use libz_sys::{
    z_stream,
    z_streamp,
    inflateInit2_,
    inflate,
    inflateEnd,
    uInt,
    voidpf
};

unsafe extern "C" fn zalloc(_: voidpf, n: uInt, c: uInt) -> voidpf {
    libc::calloc(n as usize, c as usize)
}

unsafe extern "C" fn zfree(_: voidpf, p: voidpf) {
    libc::free(p)
}

use std::io::prelude::*;
use std::io::Result;
use std::fs::File;
use std::ptr::null_mut;

struct Decoder {
    input: Box<dyn Read>,
    stream: z_stream,
}

impl Decoder {
    fn new(input: impl Read + 'static) -> Decoder {
        Decoder {
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
        Ok(0)
    }
}


#[cfg(test)]
mod tests {
    #[test]
    fn test() {
    }
}
