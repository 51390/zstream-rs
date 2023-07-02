extern crate libc;

use libz_sys::{ uInt, voidpf };

pub unsafe extern "C" fn zalloc(_: voidpf, n: uInt, c: uInt) -> voidpf {
    libc::calloc(n as usize, c as usize)
}

pub unsafe extern "C" fn zfree(_: voidpf, p: voidpf) {
    libc::free(p)
}
