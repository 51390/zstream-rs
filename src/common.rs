extern crate libc;
use libz_sys::{ uInt, voidpf };
use log::info;

static mut allocation_counter: u64 = 0;


pub unsafe extern "C" fn zalloc(_: voidpf, n: uInt, c: uInt) -> voidpf {
    allocation_counter += 1;
    info!("zalloc, counter @ {}", allocation_counter);
    libc::calloc(n as usize, c as usize)
}

pub unsafe extern "C" fn zfree(_: voidpf, p: voidpf) {
    allocation_counter -= 1;
    info!("zfree, counter @ {}", allocation_counter);
    libc::free(p)
}
