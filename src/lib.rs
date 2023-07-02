mod common;
mod decoder;
mod encoder;

pub use decoder::Decoder;
pub use encoder::Encoder;

use std::io::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{OpenOptions};
    use log::{info, warn, error};

    #[test]
    fn test_decoder() {
        let f = std::fs::OpenOptions::new().read(true).open("test/data/test.gz").unwrap();
        let mut decoder = Decoder::new(f);
        let mut output = Vec::<u8>::new();

        loop {
            let mut buffer : [u8;1024 * 1024] = [0; 1024 * 1024];
            match decoder.read(&mut buffer) {
                Ok(bytes) => {
                    //:w
                    //info!("Read: ||{}||", String::from_utf8(buffer[0..bytes].to_vec()).unwrap());
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
        let mut encoder = Encoder::new(f);
        let mut output = Vec::<u8>::new();

        loop {
            let mut buffer : [u8;1024 * 1024] = [0; 1024 * 1024];
            match encoder.read(&mut buffer) {
                Ok(bytes) => {
                    //:w
                    //info!("Read: ||{}||", String::from_utf8(buffer[0..bytes].to_vec()).unwrap());
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

        let mut output_file = OpenOptions::new().write(true).create(true).open("/tmp/out.gz").unwrap();
        output_file.write_all(output.as_slice()).unwrap();
    }
}
