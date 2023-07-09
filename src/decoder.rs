use std::io::prelude::*;
use std::io::{Result, Error, ErrorKind};
use std::ptr::null_mut;
use std::mem::size_of;
use libz_sys::{
    z_stream,
    z_streamp,
    inflateInit2_,
    inflate,
    inflateEnd,
    zlibVersion,
    Z_OK,
    Z_STREAM_END,
    Z_NO_FLUSH,
};
use log::info;

pub struct Decoder {
    input: Box<dyn Read>,
    stream: z_stream,
    initialized: bool,
    is_done: bool,
    buffer: Vec<u8>,
    bytes_in: Vec<u8>,
    bytes_out: Vec<u8>,
}

impl Decoder {
    pub fn new(input: impl Read + 'static) -> Decoder {
        Self::new_with_size(input, 128)
    }

    pub fn new_with_size(input: impl Read + 'static, size: usize) -> Decoder {
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
                zalloc: super::common::zalloc,
                zfree: super::common::zfree,
                opaque: null_mut(),
                data_type: 0,
                adler: 0,
                reserved: 0,
            },
            is_done: false,
            buffer: vec!(0; size),
            bytes_in: Vec::<u8>::new(),
            bytes_out: Vec::<u8>::new(),
        }
    }

    pub fn stream(&mut self) -> &mut z_stream {
        &mut self.stream
    }

    pub fn is_done(&self) -> bool {
        self.is_done
    }

    pub fn bytes_in(&self) -> &Vec<u8> {
        &self.bytes_in
    }

    pub fn bytes_out(&self) -> &Vec<u8> {
        &self.bytes_out
    }

    pub fn finish(&mut self) -> Result<usize> {
        self.is_done = true;
        self.cleanup();
        Ok(0)
    }

    pub fn cleanup(&mut self) {
        if self.initialized {
            unsafe { inflateEnd(&mut self.stream as z_streamp) };
        }
        self.initialized = false;
    }
}

impl Drop for Decoder {
    fn drop(&mut self) {
        info!("Decoder cleaning up");
        self.cleanup();
    }
}
impl Read for Decoder {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        if self.is_done {
            return Ok(0);
        }

        let previous_out = self.stream.total_out;
        let mut inner_buf = self.buffer.as_mut_slice();
        let bytes = match self.input.read(&mut inner_buf) {
            Ok(bytes) => {
                self.bytes_in.extend(&inner_buf[0..bytes]);
                bytes
            },
            Err(e) =>  { return Err(e); },
        };

        if bytes == 0 {
            return Ok(0);
        }

        if !self.initialized {
            self.initialized = Z_OK == unsafe {
                inflateInit2_(&mut self.stream as z_streamp, 32+15, zlibVersion(), size_of::<z_stream>() as i32)
            };

            if !self.initialized {
                return Err(Error::new(ErrorKind::Other, "Failed initializing zlib."));
            }
        }

        self.stream.next_in = inner_buf.as_mut_ptr();
        self.stream.avail_in = bytes as u32;
        self.stream.next_out = buf.as_mut_ptr();
        self.stream.avail_out = buf.len() as u32;

        let result = unsafe { inflate(&mut self.stream as z_streamp, Z_NO_FLUSH) };

        if Z_OK ==  result || Z_STREAM_END == result {
            self.is_done = Z_STREAM_END == result;
            let decompressed = self.stream.total_out - previous_out;
            self.bytes_out.extend(&buf[0..decompressed as usize]);
            Ok((decompressed) as usize)
        } else {
            let error = match result {
                libz_sys::Z_BUF_ERROR => "Z_BUFF_ERROR".to_owned(),
                libz_sys::Z_MEM_ERROR => "Z_MEM_ERROR".to_owned(),
                libz_sys::Z_STREAM_ERROR => "Z_STREAM_ERROR".to_owned(),
                libz_sys::Z_NEED_DICT => "Z_BUFF_ERROR".to_owned(),
                libz_sys::Z_DATA_ERROR => "Z_DATA_ERROR".to_owned(),
                _ =>  format!("UNKNOWN; error code {}", result),
            };

            Err(Error::new(ErrorKind::Other, format!("Failed inflating: {}", error)))
        }
    }
}


