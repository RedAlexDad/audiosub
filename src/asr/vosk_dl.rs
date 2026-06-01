#![allow(unsafe_code)]

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_float, c_int, c_void};
use std::ptr::NonNull;
use std::sync::LazyLock;

use libloading::{Library, Symbol};

static VOSK: LazyLock<Option<VoskDl>> = LazyLock::new(|| VoskDl::load().ok());

pub fn is_available() -> bool {
    VOSK.is_some()
}

pub fn load() -> Result<&'static VoskDl, Error> {
    VOSK.as_ref()
        .ok_or_else(|| Error("libvosk.so not found (install Vosk SDK)".into()))
}

pub struct VoskDl {
    lib: Library,
}

unsafe impl Send for VoskDl {}
unsafe impl Sync for VoskDl {}

impl VoskDl {
    fn load() -> Result<Self, Error> {
        let lib = unsafe { Library::new("libvosk.so") }.map_err(|e| Error(format!("libvosk.so: {e}")))?;
        Ok(Self { lib })
    }

    pub fn model_new(&self, path: &str) -> Option<Model> {
        let cpath = CString::new(path).ok()?;
        unsafe {
            let f: Symbol<unsafe extern "C" fn(*const c_char) -> *mut c_void> = self.lib.get(b"vosk_model_new").ok()?;
            let ptr = f(cpath.as_ptr());
            NonNull::new(ptr).map(Model)
        }
    }

    pub fn recognizer_new(&self, model: &Model, sample_rate: f32) -> Option<Recognizer> {
        unsafe {
            let f: Symbol<unsafe extern "C" fn(*mut c_void, c_float) -> *mut c_void> =
                self.lib.get(b"vosk_recognizer_new").ok()?;
            let ptr = f(model.0.as_ptr(), sample_rate);
            NonNull::new(ptr).map(Recognizer)
        }
    }

    pub fn recognizer_set_words(&self, rec: &Recognizer, words: bool) {
        unsafe {
            if let Ok(f) = self
                .lib
                .get::<unsafe extern "C" fn(*mut c_void, c_int)>(b"vosk_recognizer_set_words")
            {
                f(rec.0.as_ptr(), words as c_int);
            }
        }
    }

    pub fn accept_waveform(&self, rec: &Recognizer, data: &[i16]) -> DecodingState {
        unsafe {
            let f: Symbol<unsafe extern "C" fn(*mut c_void, *const i16, c_int) -> c_int> =
                match self.lib.get(b"vosk_recognizer_accept_waveform") {
                    Ok(f) => f,
                    Err(_) => return DecodingState::Running,
                };
            let ret = f(rec.0.as_ptr(), data.as_ptr(), data.len() as c_int);
            if ret == 1 {
                DecodingState::Finalized
            } else {
                DecodingState::Running
            }
        }
    }

    pub fn partial_result(&self, rec: &Recognizer) -> String {
        unsafe {
            let f: Symbol<unsafe extern "C" fn(*mut c_void) -> *const c_char> =
                match self.lib.get(b"vosk_recognizer_partial_result") {
                    Ok(f) => f,
                    Err(_) => return String::new(),
                };
            let ptr = f(rec.0.as_ptr());
            if ptr.is_null() {
                String::new()
            } else {
                CStr::from_ptr(ptr).to_string_lossy().into_owned()
            }
        }
    }

    pub fn result(&self, rec: &Recognizer) -> String {
        unsafe {
            let f: Symbol<unsafe extern "C" fn(*mut c_void) -> *const c_char> =
                match self.lib.get(b"vosk_recognizer_result") {
                    Ok(f) => f,
                    Err(_) => return String::new(),
                };
            let ptr = f(rec.0.as_ptr());
            if ptr.is_null() {
                String::new()
            } else {
                CStr::from_ptr(ptr).to_string_lossy().into_owned()
            }
        }
    }

    pub fn recognizer_reset(&self, rec: &Recognizer) {
        unsafe {
            if let Ok(f) = self
                .lib
                .get::<unsafe extern "C" fn(*mut c_void)>(b"vosk_recognizer_reset")
            {
                f(rec.0.as_ptr());
            }
        }
    }
}

pub struct Model(NonNull<c_void>);

unsafe impl Send for Model {}

impl Drop for Model {
    fn drop(&mut self) {
        if let Some(vosk) = VOSK.as_ref() {
            unsafe {
                if let Ok(f) = vosk.lib.get::<unsafe extern "C" fn(*mut c_void)>(b"vosk_model_free") {
                    f(self.0.as_ptr());
                }
            }
        }
    }
}

pub struct Recognizer(NonNull<c_void>);

unsafe impl Send for Recognizer {}

impl Drop for Recognizer {
    fn drop(&mut self) {
        if let Some(vosk) = VOSK.as_ref() {
            unsafe {
                if let Ok(f) = vosk
                    .lib
                    .get::<unsafe extern "C" fn(*mut c_void)>(b"vosk_recognizer_free")
                {
                    f(self.0.as_ptr());
                }
            }
        }
    }
}

pub enum DecodingState {
    Running,
    Finalized,
}

#[derive(Debug)]
pub struct Error(pub String);

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for Error {}
