use log::info;
use std::io::prelude::*;
use std::io::{Result, Error, ErrorKind};
use std::ptr::null_mut;
use std::mem::size_of;
use libz_sys::{
    z_stream,
    z_streamp,
    deflateInit2_,
    deflate,
    //deflateEnd,
    zlibVersion,
    Z_OK,
    Z_STREAM_END,
    //Z_NO_FLUSH,
    Z_SYNC_FLUSH,
    Z_FINISH,
};

pub struct Encoder {
    input: Box<dyn Read>,
    stream: z_stream,
    initialized: bool,
    finish: bool,
    is_done: bool,
    buffer: Vec<u8>,
    bytes_in: Vec<u8>,
    bytes_out: Vec<u8>,
}

impl Encoder {
    pub fn new(input: impl Read + 'static) -> Encoder {
        Self::new_with_size(input, 128)
    }

    pub fn new_with_size(input: impl Read + 'static, size: usize) -> Encoder {
        Encoder {
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
            finish: false,
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
    pub fn finish(&mut self, buf: &mut [u8]) -> Result<usize> {

        self.finish = true;
        self.read(buf)
    }
}

impl Read for Encoder {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let previous_out = self.stream.total_out;
        let mut inner_buf = self.buffer.as_mut_slice();
        let bytes = match self.input.read(&mut inner_buf) {
            Ok(bytes) => {
                self.bytes_in.extend(&inner_buf[0..bytes]);
                bytes
            },
            Err(e) =>  { return Err(e); },
        };

        if bytes == 0  && !self.finish {
            return Ok(0)
        }

        if !self.initialized {
            self.initialized = Z_OK == unsafe {
                deflateInit2_(
                    &mut self.stream as z_streamp,
                    9, // level
                    8, // method, Z_DEFLATED
                    31, // window bits, 15 = 2ˆ15 window size + gzip headers (16)
                    9, // mem level, MAX_MEM_LEVEL
                    0, // strategy, Z_DEFAULT_STRATEGY,
                    zlibVersion(),
                    size_of::<z_stream>() as i32)
            };

            if !self.initialized {
                return Err(Error::new(ErrorKind::Other, "Failed initializing zlib."));
            }
        }

        let flush = {
            if self.finish {
                Z_FINISH
            } else {
                Z_SYNC_FLUSH
            }
        };

        self.stream.next_in = inner_buf.as_mut_ptr();
        self.stream.avail_in = bytes as u32;
        self.stream.next_out = buf.as_mut_ptr();
        self.stream.avail_out = buf.len() as u32;

        let result = unsafe { deflate(&mut self.stream as z_streamp, flush) };

        if Z_OK ==  result || Z_STREAM_END == result {
            self.is_done = Z_STREAM_END == result;
            let compressed = self.stream.total_out - previous_out;
            info!(">> Read {} bytes from file, compressed {} bytes", bytes, compressed);
            self.bytes_out.extend(&buf[0..compressed as usize]);
            Ok((compressed) as usize)
        } else {
            let error = match result {
                libz_sys::Z_BUF_ERROR => "Z_BUFF_ERROR".to_owned(),
                libz_sys::Z_MEM_ERROR => "Z_MEM_ERROR".to_owned(),
                libz_sys::Z_STREAM_ERROR => "Z_STREAM_ERROR".to_owned(),
                libz_sys::Z_NEED_DICT => "Z_BUFF_ERROR".to_owned(),
                libz_sys::Z_DATA_ERROR => "Z_DATA_ERROR".to_owned(),
                _ =>  format!("UNKNOWN; error code {}", result),
            };

            Err(Error::new(ErrorKind::Other, format!("Failed deflating: {}", error)))
        }
    }

}
