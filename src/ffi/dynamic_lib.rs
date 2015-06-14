// Copyright 2013-2015 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Dynamic library facilities. Pulled from libstd since it's currently marked as unstable in that
//! context. Tweaked to remove use of unstable features.
//!
//! A simple wrapper over the platform's dynamic library facilities

#![allow(missing_docs)]

use std::env;
use std::ffi::{CString, OsString};
use std::mem;
use std::path::{Path, PathBuf};

/// An unsafe version modified version of UnsafeDynamicLibrary. Unsafe because thread safety is not
/// guaranteed on all platforms. If not limited to a single thread per-process, undefined beahvior
/// may result. Should only be used until a stable version of UnsafeDynamicLibrary lands.
pub struct UnsafeDynamicLibrary {
    handle: *mut u8
}

impl Drop for UnsafeDynamicLibrary {
    fn drop(&mut self) {
        match dl::check_for_errors_in(|| {
            unsafe {
                dl::close(self.handle)
            }
        }) {
            Ok(()) => {},
            Err(str) => panic!("{}", str)
        }
    }
}

impl UnsafeDynamicLibrary {
    // FIXME (#12938): Until DST lands, we cannot decompose &str into
    // & and str, so we cannot usefully take ToCStr arguments by
    // reference (without forcing an additional & around &str). So we
    // are instead temporarily adding an instance for &Path, so that
    // we can take ToCStr as owned. When DST lands, the &Path instance
    // should be removed, and arguments bound by ToCStr should be
    // passed by reference. (Here: in the `open` method.)

    /// Lazily open a dynamic library. When passed None it gives a
    /// handle to the calling process
    pub unsafe fn open(filename: Option<&Path>) -> Result<UnsafeDynamicLibrary, String> {
        let maybe_library = dl::open(filename.map(|path| path.as_os_str()));

        // The dynamic library must not be constructed if there is
        // an error opening the library so the destructor does not
        // run.
        match maybe_library {
            Err(err) => Err(err),
            Ok(handle) => Ok(UnsafeDynamicLibrary { handle: handle })
        }
    }

    /// Prepends a path to this process's search path for dynamic libraries
    pub unsafe fn prepend_search_path(path: &Path) {
        let mut search_path = UnsafeDynamicLibrary::search_path();
        search_path.insert(0, path.to_path_buf());
        env::set_var(UnsafeDynamicLibrary::envvar(), &UnsafeDynamicLibrary::create_path(&search_path));
    }

    /// From a slice of paths, create a new vector which is suitable to be an
    /// environment variable for this platforms dylib search path.
    pub unsafe fn create_path(path: &[PathBuf]) -> OsString {
        let mut newvar = OsString::new();
        for (i, path) in path.iter().enumerate() {
            if i > 0 { newvar.push(UnsafeDynamicLibrary::separator()); }
            newvar.push(path);
        }
        return newvar;
    }

    /// Returns the environment variable for this process's dynamic library
    /// search path
    pub unsafe fn envvar() -> &'static str {
        if cfg!(windows) {
            "PATH"
        } else if cfg!(target_os = "macos") {
            "DYLD_LIBRARY_PATH"
        } else {
            "LD_LIBRARY_PATH"
        }
    }

    fn separator() -> &'static str {
        if cfg!(windows) { ";" } else { ":" }
    }

    /// Returns the current search path for dynamic libraries being used by this
    /// process
    pub unsafe fn search_path() -> Vec<PathBuf> {
        match env::var_os(UnsafeDynamicLibrary::envvar()) {
            Some(var) => env::split_paths(&var).collect(),
            None => Vec::new(),
        }
    }

    /// Access the value at the symbol of the dynamic library
    pub unsafe fn symbol<T>(&self, symbol: &str) -> Result<*mut T, String> {
        // This function should have a lifetime constraint of 'a on
        // T but that feature is still unimplemented

        let raw_string = CString::new(symbol).unwrap();
        let maybe_symbol_value = dl::check_for_errors_in(|| {
            dl::symbol(self.handle, raw_string.as_ptr())
        });

        // The value must not be constructed if there is an error so
        // the destructor does not run.
        match maybe_symbol_value {
            Err(err) => Err(err),
            Ok(symbol_value) => Ok(mem::transmute(symbol_value))
        }
    }
}

#[cfg(any(target_os = "linux",
          target_os = "android",
          target_os = "macos",
          target_os = "ios",
          target_os = "freebsd",
          target_os = "dragonfly",
          target_os = "bitrig",
          target_os = "openbsd"))]
mod dl {
    use std::ffi::{CStr, CString, OsStr};
    use std::str;
    use libc;
    use std::ptr;

    pub fn open(filename: Option<&OsStr>) -> Result<*mut u8, String> {
        check_for_errors_in(|| {
            unsafe {
                match filename {
                    Some(filename) => open_external(filename),
                    None => open_internal(),
                }
            }
        })
    }

    const LAZY: libc::c_int = 1;

    unsafe fn open_external(filename: &OsStr) -> *mut u8 {
        let s = CString::new(filename.to_str().unwrap()).unwrap();
        dlopen(s.as_ptr(), LAZY) as *mut u8
    }

    unsafe fn open_internal() -> *mut u8 {
        dlopen(ptr::null(), LAZY) as *mut u8
    }

    // NOTE: Thread-safety code was removed here because StaticMutex is also unstable. The safety
    // is somewhat artificial in any case, since there's no guarantee external code isn't
    // potentially manipulating the dlerror() state at the same time. It does make dynamic_lib
    // itself unsafe.
    pub fn check_for_errors_in<T, F>(f: F) -> Result<T, String> where
        F: FnOnce() -> T,
    {
        unsafe {
            let _old_error = dlerror();

            let result = f();

            let last_error = dlerror() as *const _;
            let ret = if ptr::null() == last_error {
                Ok(result)
            } else {
                let s = CStr::from_ptr(last_error).to_bytes();
                Err(str::from_utf8(s).unwrap().to_string())
            };

            ret
        }
    }

    pub unsafe fn symbol(handle: *mut u8,
                         symbol: *const libc::c_char) -> *mut u8 {
        dlsym(handle as *mut libc::c_void, symbol) as *mut u8
    }
    pub unsafe fn close(handle: *mut u8) {
        dlclose(handle as *mut libc::c_void); ()
    }

    extern {
        fn dlopen(filename: *const libc::c_char,
                  flag: libc::c_int) -> *mut libc::c_void;
        fn dlerror() -> *mut libc::c_char;
        fn dlsym(handle: *mut libc::c_void,
                 symbol: *const libc::c_char) -> *mut libc::c_void;
        fn dlclose(handle: *mut libc::c_void) -> libc::c_int;
    }
}

#[cfg(target_os = "windows")]
mod dl {
    use std::ffi::OsStr;
    use std::option::Option::{self, Some, None};
    use std::os::windows::ffi::OsStrExt;
    use std::ptr;
    use std::result::Result;
    use std::result::Result::{Ok, Err};
    use std::string::String;
    use std::vec::Vec;
    use winapi::*;
    use kernel32::*;

    pub fn open(filename: Option<&OsStr>) -> Result<*mut u8, String> {
        unsafe {
            SetLastError(0);
        }

        let result = match filename {
            Some(filename) => {
                let filename_str: Vec<_> =
                    to_wide(filename);
                let result = unsafe {
                    LoadLibraryW(filename_str.as_ptr())
                };
                // beware: Vec/String may change errno during drop!
                // so we get error here.
                if result == ptr::null_mut() {
                    Err(String::from("LoadLibraryW failed"))
                } else {
                    Ok(result as *mut u8)
                }
            }
            None => {
                let mut handle = ptr::null_mut();
                let succeeded = unsafe {
                    GetModuleHandleExW(0, ptr::null(), &mut handle)
                };
                if succeeded == FALSE {
                    Err(String::from("GetModuleHandleExW failed"))
                } else {
                    Ok(handle as *mut u8)
                }
            }
        };
        result
    }

    pub fn check_for_errors_in<T, F>(f: F) -> Result<T, String> where
        F: FnOnce() -> T,
    {
        unsafe {
            SetLastError(0);

            let result = f();

            let error = errno();
            if 0 == error {
                Ok(result)
            } else {
                Err(format!("Error code {}", error))
            }
        }
    }

    pub unsafe fn symbol(handle: *mut u8, symbol: LPCSTR) -> *mut u8 {
        GetProcAddress(handle as HMODULE, symbol) as *mut u8
    }
    pub unsafe fn close(handle: *mut u8) {
        FreeLibrary(handle as HMODULE); ()
    }

    pub unsafe fn errno() -> u32 {
        GetLastError()
    }

    pub fn to_wide(s: &OsStr) -> Vec<u16> {
        s.encode_wide().collect()
    }
}
