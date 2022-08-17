use std::ptr;
use std::ffi::*;
use crate::prelude::*;

use winapi::um::fileapi::*;
use winapi::um::handleapi::*;
use winapi::um::winnt::*;
use winapi::um::memoryapi::*;

use super::io::*;

#[cfg(feature="gol-internal")]
pub fn debug_platform_read_entire_file(thread_context: &mut ThreadContext, file_name: &str) -> Option<DebugReadFileResult> {
    
    unsafe  {
        let c_file_name = CString::new(build_data_file_name(file_name)).unwrap();

        let file_handle = CreateFileA(
            c_file_name.as_ptr() as *const i8, 
            GENERIC_READ, 
            FILE_SHARE_READ, 
            ptr::null_mut(), 
            OPEN_EXISTING, 
            0, 
            ptr::null_mut());

        let mut result = None;

        if !file_handle.is_null() {
            let mut file_size = LARGE_INTEGER::default();

            if GetFileSizeEx(file_handle, &mut file_size) != 0 {
                let file_size_32 = safe_truncate_u64(*file_size.QuadPart() as u64);

                let contents = VirtualAlloc(
                    ptr::null_mut(), 
                    file_size_32 as usize, 
                    MEM_RESERVE|MEM_COMMIT, 
                    PAGE_READWRITE
                );

                if !contents.is_null() {
                    let mut bytes_read: u32 = 0;
                    
                    if ReadFile(
                        file_handle, 
                        contents, 
                        file_size_32, 
                        &mut bytes_read, 
                        ptr::null_mut()
                    ) != 0 && file_size_32 == bytes_read {
                        result = Some(
                            DebugReadFileResult {
                                contents_size: file_size_32,
                                contents,
                            }
                        );
                    } else {                    
                        debug_platform_free_file_memory(thread_context, contents);
                    }

                } else {
                }
            } else {
            }

            CloseHandle(file_handle);

        } else {
        }

        result
    }
}

#[cfg(feature="gol-internal")]
pub fn debug_platform_write_entire_file(_thread_context: &mut ThreadContext, file_name: &str, memory_size: u32, memory: *mut c_void) -> bool {
    unsafe  {
        let mut result = false;
        let c_file_name = CString::new(build_data_file_name(file_name)).unwrap();

        let file_handle = CreateFileA(
            c_file_name.as_ptr() as *const i8, 
            GENERIC_WRITE, 
            0, 
            ptr::null_mut(), 
            CREATE_ALWAYS, 
            0, 
            ptr::null_mut());

        if !file_handle.is_null() {                
            let mut bytes_written: u32 = 0;
                
            if WriteFile(file_handle, memory, memory_size, &mut bytes_written, ptr::null_mut()) != 0 {
                result = bytes_written == memory_size;
            } else {
            }

            CloseHandle(file_handle);

        } else {
        }

        return result;
    }
}

#[cfg(feature="gol-internal")]
pub fn debug_platform_free_file_memory(_thread_context: &mut ThreadContext, memory: *mut c_void) {
    unsafe {
        if !memory.is_null() {
            VirtualFree(memory, 0, MEM_RELEASE);
        }
    }
}