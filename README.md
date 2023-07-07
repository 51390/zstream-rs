# zstream-rs
A flexible implementation of gzip stream decoder/encoder for Rust.

## Why

Some encoder/decoder crates that provide Read Traits exhibit a behavior of _eagerly_ trying to consume stream data from the underlying readers that are passed to them.
For use cases where the underlying stream data is not fully available for encoding/decoding immediately, this may lead to such implementations breaking with "corrupt deflate stream" errors or similar.

## How

This create makes no assumptions about the underlying stream data availability, and will not call the libz functions for neither inflating nor deflating streaming data unless there are new bytes available for such.
This allows a better flow control of the encoding / decoding process, in such cases where data may take a longer time to reach the component that is performing the inflate / deflate process.
