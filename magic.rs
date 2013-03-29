#[link(name = "magic",
       vers = "0.1.0",
       uuid = "201abfd3-0d07-41ab-a6c5-9eb94b318383",
       url = "https://github.com/thestinger/rust-magic")];

#[comment = "libmagic bindings"];
#[license = "MIT"];
#[crate_type = "lib"];

extern mod std;

use core::libc::{c_char, c_int, size_t};
use core::ptr::is_null;
use core::str::as_c_str;

enum Magic {}

pub enum MagicFlag {
    /// No flags
    MAGIC_NONE              = 0x000000,
    /// Turn on debugging
    MAGIC_DEBUG             = 0x000001,
    /// Follow symlinks
    MAGIC_SYMLINK           = 0x000002,
    /// Check inside compressed files
    MAGIC_COMPRESS          = 0x000004,
    /// Look at the contents of devices
    MAGIC_DEVICES           = 0x000008,
    /// Return the MIME type
    MAGIC_MIME_TYPE         = 0x000010,
    /// Return all matches
    MAGIC_CONTINUE          = 0x000020,
    /// Print warnings to stderr
    MAGIC_CHECK             = 0x000040,
    /// Restore access time on exit
    MAGIC_PRESERVE_ATIME    = 0x000080,
    /// Don't translate unprintable chars
    MAGIC_RAW               = 0x000100,
    /// Handle ENOENT etc as real errors
    MAGIC_ERROR             = 0x000200,
    /// Return the MIME encoding
    MAGIC_MIME_ENCODING     = 0x000400,
    /// `MAGIC_MIME_TYPE` and `MAGIC_MIME_ENCODING`
    MAGIC_MIME              = 0x000410,
    /// Return the Apple creator and type
    MAGIC_APPLE             = 0x000800,
    /// Don't check for compressed files
    MAGIC_NO_CHECK_COMPRESS = 0x001000,
    /// Don't check for tar files
    MAGIC_NO_CHECK_TAR      = 0x002000,
    /// Don't check magic entries
    MAGIC_NO_CHECK_SOFT     = 0x004000,
    /// Don't check application type
    MAGIC_NO_CHECK_APPTYPE  = 0x008000,
    /// Don't check for elf details
    MAGIC_NO_CHECK_ELF      = 0x010000,
    /// Don't check for text files
    MAGIC_NO_CHECK_TEXT     = 0x020000,
    /// Don't check for cdf files
    MAGIC_NO_CHECK_CDF      = 0x040000,
    /// Don't check tokens
    MAGIC_NO_CHECK_TOKENS   = 0x100000,
    /// Don't check text encodings
    MAGIC_NO_CHECK_ENCODING = 0x200000,
}

fn combine_flags(flags: &[MagicFlag]) -> c_int {
    vec::foldl(0, flags, |a: c_int, b: &MagicFlag| a | (*b as c_int))
}

#[link_args = "-lmagic"]
extern "C" {
    fn magic_open(flags: c_int) -> *Magic;
    fn magic_close(cookie: *Magic);
    fn magic_error(cookie: *Magic) -> *c_char;
    fn magic_errno(cookie: *Magic) -> c_int;
    fn magic_descriptor(cookie: *Magic, fd: c_int) -> *c_char;
    fn magic_file(cookie: *Magic, filename: *c_char) -> *c_char;
    fn magic_buffer(cookie: *Magic, buffer: *u8, length: size_t) -> *c_char;
    fn magic_setflags(cookie: *Magic, flags: c_int) -> c_int;
    fn magic_check(cookie: *Magic, filename: *c_char) -> c_int;
    fn magic_compile(cookie: *Magic, filename: *c_char) -> c_int;
    fn magic_list(cookie: *Magic, filename: *c_char) -> c_int;
    fn magic_load(cookie: *Magic, filename: *c_char) -> c_int;
}

pub struct Cookie {
    priv cookie: *Magic,
}

impl Drop for Cookie {
    fn finalize(&self) { unsafe { magic_close(self.cookie) } }
}

impl Cookie {
    fn file(&self, filename: &str) -> Option<~str> {
        unsafe {
            let cookie = self.cookie;
            let s = as_c_str(filename, |filename| magic_file(cookie, filename));
            if is_null(s) { None } else { Some(str::raw::from_c_str(s)) }
        }
    }

    fn buffer(&self, buffer: &[u8]) -> Option<~str> {
        unsafe {
            let buffer_len = buffer.len() as size_t;
            let pbuffer = vec::raw::to_ptr(buffer);
            let s = magic_buffer(self.cookie, pbuffer, buffer_len);
            if is_null(s) { None } else { Some(str::raw::from_c_str(s)) }
        }
    }

    fn error(&self) -> Option<~str> {
        unsafe {
            let s = magic_error(self.cookie);
            if is_null(s) { None } else { Some(str::raw::from_c_str(s)) }
        }
    }

    fn setflags(&self, flags: &[MagicFlag]) {
        unsafe {
            magic_setflags(self.cookie, combine_flags(flags));
        }
    }

    fn check(&self, filename: &str) -> bool {
        unsafe {
            let cookie = self.cookie;
            as_c_str(filename, |filename| magic_check(cookie, filename)) == 0
        }
    }

    fn compile(&self, filename: &str) -> bool {
        unsafe {
            let cookie = self.cookie;
            as_c_str(filename, |filename| magic_compile(cookie, filename)) == 0
        }
    }

    fn list(&self, filename: &str) -> bool {
        unsafe {
            let cookie = self.cookie;
            as_c_str(filename, |filename| magic_list(cookie, filename)) == 0
        }
    }

    fn load(&self, filename: &str) -> bool {
        unsafe {
            let cookie = self.cookie;
            as_c_str(filename, |filename| magic_load(cookie, filename)) == 0
        }
    }

    static fn open(flags: &[MagicFlag]) -> Option<Cookie> {
        unsafe {
            let cookie = magic_open(combine_flags(flags));
            if is_null(cookie) { None } else { Some(Cookie{cookie: cookie,}) }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file() {
        let cookie = Cookie::open([MAGIC_NONE]).unwrap();
        fail_unless!(cookie.load("/usr/share/file/misc/magic.mgc"));

        fail_unless!(cookie.file("rust-logo-128x128-blk.png").unwrap() ==
            ~"PNG image data, 128 x 128, 8-bit/color RGBA, non-interlaced");

        cookie.setflags([MAGIC_MIME_TYPE]);
        fail_unless!(cookie.file("rust-logo-128x128-blk.png").unwrap() ==
            ~"image/png");

        cookie.setflags([MAGIC_MIME_TYPE, MAGIC_MIME_ENCODING]);
        fail_unless!(cookie.file("rust-logo-128x128-blk.png").unwrap() ==
            ~"image/png; charset=binary");
    }

    #[test]
    fn buffer() {
        let cookie = Cookie::open([MAGIC_NONE]).unwrap();
        fail_unless!(cookie.load("/usr/share/file/misc/magic.mgc"));

        let s = ~"#!/usr/bin/env python3\nprint('Hello, world!')";
        fail_unless!(str::as_bytes(&s, |bytes| {
          cookie.buffer(*bytes)
        }).unwrap() == ~"Python script, ASCII text executable");

        cookie.setflags([MAGIC_MIME_TYPE]);
        fail_unless!(str::as_bytes(&s, |bytes| {
          cookie.buffer(*bytes)
        }).unwrap() == ~"text/x-python");
    }

    #[test]
    fn file_error() {
        let cookie = Cookie::open([MAGIC_NONE]).unwrap();
        fail_unless!(cookie.load("/usr/share/file/misc/magic.mgc"));

        let ret = cookie.file("non-existent_file.txt");
        fail_unless!(ret.is_none());
        fail_unless!(cookie.error().unwrap() ==
            ~"cannot open `non-existent_file.txt' (No such file or directory)");
    }
}