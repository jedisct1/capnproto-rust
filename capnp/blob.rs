/*
 * Copyright (c) 2013, David Renshaw (dwrenshaw@gmail.com)
 *
 * See the LICENSE file in the capnproto-rust root directory.
 */

pub mod Text {
    use std;

    pub type Reader<'a> = &'a str;

    // len does not include the required null terminator at the end
    pub fn new_reader<'a>(p : *u8, len : uint) -> Reader<'a> {
        unsafe {
            let v = std::unstable::raw::Slice { data: p, len: len };
            assert!(std::str::is_utf8(std::cast::transmute(v)));
            std::cast::transmute(v)
        }
    }

    pub struct Builder<'a> {
        priv ptr : *mut u8,
        priv len : uint,
    }

    impl <'a> Builder <'a> {

        pub fn new<'b>(p : *mut u8, len : uint) -> Builder<'b> {
            Builder { ptr : p, len : len}
        }

        pub fn as_mut_bytes(&self) -> &'a mut [u8] {
             unsafe { std::cast::transmute(std::unstable::raw::Slice { data:self.ptr as *u8, len: self.len }) }
        }

    }

}

pub mod Data {
    use std;

    pub type Reader<'a> = &'a [u8];

    pub fn new_reader<'a>(p : *u8, len : uint) -> Reader<'a> {
        unsafe {
            let v = std::unstable::raw::Slice { data: p, len: len };
            std::cast::transmute(v)
        }
    }

    pub type Builder<'a> = &'a mut [u8];

    pub fn new_builder<'a>(p : *mut u8, len : uint) -> Builder<'a> {
        unsafe {
            let v = std::unstable::raw::Slice { data: p as *u8, len: len };
            std::cast::transmute(v)
        }
    }

}