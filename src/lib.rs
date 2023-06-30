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
    Z_OK,
    Z_STREAM_END,
    Z_NO_FLUSH,
    Z_SYNC_FLUSH,
};

use std::io::prelude::*;
use std::io::{Result, Error, ErrorKind};
use std::ptr::null_mut;
use std::mem::size_of;

unsafe extern "C" fn zalloc(_: voidpf, n: uInt, c: uInt) -> voidpf {
    libc::calloc(n as usize, c as usize)
}

unsafe extern "C" fn zfree(_: voidpf, p: voidpf) {
    libc::free(p)
}

pub struct Decoder {
    input: Box<dyn Read>,
    stream: z_stream,
    initialized: bool,
    is_done: bool,
    buffer: Vec<u8>,
    buffer_size: usize,
}

impl Decoder {
    pub fn new(input: impl Read + 'static) -> Decoder {
        Self::new_with_size(input, 1024)
    }

    pub fn new_with_size(input: impl Read + 'static, size: usize) -> Decoder {
        let buffer = Vec::<u8>::new();
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
            is_done: false,
            buffer_size: size,
            buffer: vec!(0; size),
        }
    }

    pub fn stream(&mut self) -> &mut z_stream {
        &mut self.stream
    }

    pub fn is_done(&self) -> bool {
        self.is_done
    }
}

impl Read for Decoder {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let previous_out = self.stream.total_out;
        let mut inner_buf = self.buffer.as_mut_slice();
        let bytes = match self.input.read(&mut inner_buf) {
            Ok(bytes) => bytes,
            Err(e) =>  { return Err(e); },
        };

        if bytes == 0 {
            return Ok(0);
        }

        if !self.initialized {
            self.initialized = Z_OK == unsafe {
                inflateInit2_(&mut self.stream as z_streamp, 16+15, zlibVersion(), size_of::<z_stream>() as i32)
            };

            if !self.initialized {
                return Err(Error::new(ErrorKind::Other, "Failed initializing zlib."));
            }
        }

        self.stream.next_in = inner_buf.as_mut_ptr();
        self.stream.avail_in = bytes as u32;
        self.stream.next_out = buf.as_mut_ptr();
        self.stream.avail_out = buf.len() as u32;

        let result = unsafe { inflate(&mut self.stream as z_streamp, Z_SYNC_FLUSH) };

        if Z_OK ==  result || Z_STREAM_END == result {
            self.is_done = Z_STREAM_END == result;
            let decompressed = self.stream.total_out - previous_out;
            println!(">> Read {} bytes from file, decompressed {} bytes", bytes, decompressed);
            Ok((decompressed) as usize)
        } else {
            let error = match result {
                libz_sys::Z_BUF_ERROR => "Z_BUFF_ERROR".to_owned(),
                libz_sys::Z_MEM_ERROR => "Z_MEM_ERROR".to_owned(),
                libz_sys::Z_STREAM_ERROR => "Z_STREAM_ERROR".to_owned(),
                libz_sys::Z_NEED_DICT => "Z_BUFF_ERROR".to_owned(),
                _ =>  format!("UNKNOWN; error code {}", result),
            };

            Err(Error::new(ErrorKind::Other, format!("Failed inflating: {}", error)))
        }
    }

}


#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{OpenOptions};
    use log::{info, warn, error};

    #[test]
    fn test() {
        let f = std::fs::OpenOptions::new().read(true).open("test/data/test.gz").unwrap();
        let mut decoder = Decoder::new(f);
        let mut output = Vec::<u8>::new();

        loop {
            let mut buffer : [u8;1024 * 1024] = [0; 1024 * 1024];
            match decoder.read(&mut buffer) {
                Ok(bytes) => {
                    //:w
                    //println!("Read: ||{}||", String::from_utf8(buffer[0..bytes].to_vec()).unwrap());
                    if bytes > 0 {
                        output.extend(&buffer[0..bytes]);
                    }

                    if decoder.is_done {
                        break;
                    }
                },
                Err(e) => {
                    error!("{}", e);
                    assert!(false, "{}", e);
                }
            }
        }

        let mut output_file = OpenOptions::new().write(true).create(true).open("/tmp/out.txt").unwrap();
        output_file.write_all(output.as_slice()).unwrap();
    }
}
