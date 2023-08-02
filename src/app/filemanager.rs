//todo: make it more dependency-agnostic
use symphonia::core::io::MediaSource;

pub trait FileManager {
    //required to convert into symphonia::MediaSource

    type Stream: MediaSource;
    type Error: std::error::Error;
    fn get_file(&self, path: impl ToString) -> Result<Self::Stream, Self::Error>;
}
#[cfg(not(feature = "web"))]

mod native {
    use super::*;
    pub struct NativeFileManager {
        // currently has no member
    }
    impl FileManager for NativeFileManager {
        type Stream = std::fs::File;
        type Error = std::io::Error;
        fn get_file(&self, path: impl ToString) -> Result<Self::Stream, Self::Error> {
            let s = path.to_string();
            let p = std::path::Path::new(&s);
            std::fs::File::open(p)
        }
    }
}
#[cfg(feature = "web")]

pub mod web {
    use super::*;
    pub struct WebMediaSource {}
    impl std::io::Read for WebMediaSource {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            todo!()
        }
    }
    impl std::io::Seek for WebMediaSource {
        fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
            todo!()
        }
    }
    unsafe impl Send for WebMediaSource {}
    unsafe impl Sync for WebMediaSource {}
    impl MediaSource for WebMediaSource {
        fn is_seekable(&self) -> bool {
            todo!()
        }

        fn byte_len(&self) -> Option<u64> {
            todo!()
        }
    }
    pub struct WebFileManager {}

    pub struct WebFileError {}
    impl std::error::Error for WebFileError {}
    impl std::fmt::Display for WebFileError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "file operation is currenrly not compatible with wasm")
        }
    }

    impl std::fmt::Debug for WebFileError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("WebFileError").finish()
        }
    }

    impl FileManager for WebFileManager {
        type Stream = WebMediaSource;
        type Error = WebFileError;
        fn get_file(&self, path: impl ToString) -> Result<Self::Stream, Self::Error> {
            Err(WebFileError {})
        }
    }
}

#[cfg(not(feature = "web"))]

static GLOBAL_FILE_MANAGER: native::NativeFileManager = native::NativeFileManager {};
#[cfg(feature = "web")]

static GLOBAL_FILE_MANAGER: web::WebFileManager = web::WebFileManager {};

pub fn get_global_file_manager() -> &'static impl FileManager {
    &GLOBAL_FILE_MANAGER
}
