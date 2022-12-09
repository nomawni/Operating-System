#![no_std]
#![no_main]
use sys_call as sys;
use user_shared::*;

#[no_mangle]
extern "C" fn main() {
    sys::exit();
}
