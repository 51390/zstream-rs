mod common;
mod decoder;
mod encoder;

pub use decoder::Decoder;
pub use encoder::Encoder;

#[cfg(test)]
mod tests {
    use log::{error};
    use std::fs::{OpenOptions};
    use std::io::prelude::*;
    use super::*;

    #[test]
    fn test_decoder() {
        let f = std::fs::OpenOptions::new().read(true).open("test/data/main.js.gz").unwrap();
        let mut decoder = Decoder::new(f);
        let mut output = Vec::<u8>::new();

        loop {
            let mut buffer : [u8;1024 * 1024] = [0; 1024 * 1024];
            match decoder.read(&mut buffer) {
                Ok(bytes) => {
                    if bytes > 0 {
                        output.extend(&buffer[0..bytes]);
                    }

                    if decoder.is_done() {
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

    #[test]
    fn test_encoder() {
        let f = std::fs::OpenOptions::new().read(true).open("test/data/test.txt").unwrap();
        let mut encoder = Encoder::new(Box::new(f));
        let mut buffer : [u8;1024 * 1024] = [0; 1024 * 1024];
        let mut output = Vec::<u8>::new();

        loop {
            match encoder.read(&mut buffer) {
                Ok(bytes) => {
                    if bytes > 0 {
                        output.extend(&buffer[0..bytes]);
                    } else {
                        break;
                    }

                    if encoder.is_done() {
                        break;
                    }
                },
                Err(e) => {
                    error!("{}", e);
                    assert!(false, "{}", e);
                }
            }
        }

        match encoder.finish(&mut buffer)  {
            Ok(bytes) => {
                if bytes > 0 {
                    output.extend(&buffer[0..bytes]);
                }
            },
            Err(e) => {
                error!("{}", e);
                assert!(false, "{}", e);
            }
        }


        let mut output_file = OpenOptions::new().write(true).create(true).open("/tmp/out.gz").unwrap();
        output_file.write_all(output.as_slice()).unwrap();
    }

    #[test]
    fn test_encode_decode() {

        struct TestReader {
            test_data: Vec<u8>,
            cursor: usize,
        }

        impl Read for TestReader {
            fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
                let n = std::cmp::min(buf.len(), self.test_data.len() - self.cursor);
                if n > 0 {
                    buf[0..n].copy_from_slice(&self.test_data[self.cursor..self.cursor + n]);
                    self.cursor += n;
                    println!("{} bytes from test data consumed, cursor is now at {}. Total {} bytes.", n, self.cursor, self.test_data.len());
                    Ok(n)
                } else {
                    Ok(0)
                }
            }
        }

        let input: Vec<u8> = vec!(0; 256);
        let mut output = Vec::<u8>::new();
        let mut encoder = Encoder::new(TestReader { test_data: input.clone(), cursor: 0 });
        let mut buffer : [u8; 128 * 1024] = [0; 128 * 1024];

        loop {
            // FIXME: improve this loop by not assuming the encoder will only
            // return 0 bytes at the end of the stream. It may return 0 bytes
            // prior to having finished consumeing the input buffer.
            match encoder.read(&mut buffer) {
                Ok(bytes) => {
                    if bytes > 0 {
                        output.extend(&buffer[0..bytes]);
                    } else {
                        break;
                    }

                    if encoder.is_done() {
                        break;
                    }
                },
                Err(e) => {
                    error!("{}", e);
                    assert!(false, "{}", e);
                }
            }
        }

        match encoder.finish(&mut buffer)  {
            Ok(bytes) => {
                println!("Encoded {} bytes (finish).", bytes);
                if bytes > 0 {
                    output.extend(&buffer[0..bytes]);
                }
            },
            Err(e) => {
                error!("{}", e);
                assert!(false, "{}", e);
            }
        }

        println!("Output buffer len: {}", output.len());

        let encoded = TestReader { test_data: output.clone(), cursor: 0 };
        let mut decoder = Decoder::new(encoded);
        output.clear();

        loop {
            match decoder.read(&mut buffer) {
                Ok(bytes) => {
                    println!("Decoded {} bytes.", bytes);
                    if bytes > 0 {
                        output.extend(&buffer[0..bytes]);
                    }

                    if decoder.is_done() {
                        break;
                    }
                },
                Err(e) => {
                    error!("{}", e);
                    assert!(false, "{}", e);
                }
            }
        }

        assert_eq!(input.len(), output.len());
        assert_eq!(input, output);
    }
}
