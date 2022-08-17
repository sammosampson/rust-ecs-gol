use std::{
    ffi::*,
    ptr,
    mem::*
};

use winapi::um::{
    winnt::*,
    fileapi::*,
    handleapi::*,
    winbase::*,
    memoryapi::*
};

use super::io::*;
use crate::prelude::*;

pub struct Win32ReplayBuffer {
    pub file_handle: HANDLE,
    pub memory_map: HANDLE,
    pub file_name: String,
    pub memory_block: *mut c_void
}

impl Default for Win32ReplayBuffer {
    fn default() -> Self {
        Self { 
            file_handle: ptr::null_mut(),
            memory_map: ptr::null_mut(),
            file_name: Default::default(),
            memory_block: ptr::null_mut()
        }
    }
}

impl Win32ReplayBuffer {
    fn new(replay_index: usize, total_size: usize) -> Self {
        
        let file_name = get_input_file_location(replay_index, false);
        let c_file_name = CString::new(&*file_name).unwrap();
        unsafe {
            let file_handle = CreateFileA(
                c_file_name.as_ptr() as *const i8, 
                GENERIC_WRITE|GENERIC_READ, 
                0, 
                ptr::null_mut(), 
                CREATE_ALWAYS, 
                0, 
                ptr::null_mut()
            );

            let mut max_size = LARGE_INTEGER::default();
            *max_size.QuadPart_mut() = total_size as i64;

            let memory_map = CreateFileMappingA(
                file_handle, 
                ptr::null_mut(), 
                PAGE_READWRITE, 
                max_size.u().HighPart as u32,
                    max_size.u().LowPart as u32, 
                    ptr::null_mut()
            );

            let memory_block = MapViewOfFile(
                memory_map, 
                FILE_MAP_ALL_ACCESS,
                0, 
                0, 
                total_size
            );

            if memory_block.is_null() {
                println!("cannot initialise loop edit file");
            }

            Self {
                file_handle,
                file_name,
                memory_block,
                memory_map
            }
        }
    }
}

pub struct Win32State {
    recording_handle: Option<HANDLE>,
    pub replay_buffers: [Win32ReplayBuffer; 4],
    input_recording_index: usize,
    playback_handle: Option<HANDLE>,
    input_playing_index: usize
}

impl Win32State {
    pub fn new() -> Self {
        Self {
            recording_handle: None,
            replay_buffers: [
                Win32ReplayBuffer::default(),
                Win32ReplayBuffer::new(1, get_total_allocated_memory_size()),
                Win32ReplayBuffer::new(2, get_total_allocated_memory_size()),
                Win32ReplayBuffer::new(3, get_total_allocated_memory_size())
            ],
            input_recording_index: 0,
            playback_handle: None,
            input_playing_index: 0
        }
    }

    pub fn is_recording(&self) -> bool {
        self.input_recording_index > 0
    }

    pub fn is_playing(&self) -> bool {
        self.input_playing_index > 0
    }

    pub fn begin_recording_input(&mut self, input_recording_index: usize) {
        self.input_recording_index = input_recording_index;   
    
        let file_name = CString::new(get_input_file_location(input_recording_index, true)).unwrap();
            
        unsafe { 
            self.recording_handle = Some(CreateFileA(
                file_name.as_ptr() as *const i8, 
                GENERIC_WRITE, 
                0, 
                ptr::null_mut(), 
                CREATE_ALWAYS, 
                0, 
                ptr::null_mut())
            );

            if cfg!(feature = "0") {
                let mut file_position = LARGE_INTEGER::default();
                *file_position.QuadPart_mut() = get_total_allocated_memory_size() as i64;
                SetFilePointerEx(self.recording_handle.unwrap(), file_position, ptr::null_mut(), FILE_BEGIN);
            }

            let replay_buffer = self.get_replay_buffer(input_recording_index);
            RtlCopyMemory(replay_buffer.memory_block, get_allocated_memory_block(), get_total_allocated_memory_size());   
        }
    }    

    fn get_replay_buffer(&self, index: usize) -> &Win32ReplayBuffer {
        gol_assert!(index < self.replay_buffers.len());
        &self.replay_buffers[index]
    }

    pub fn end_recording_input(&mut self) {
        unsafe {
            CloseHandle(self.recording_handle.unwrap());
            self.recording_handle = None;
            self.input_recording_index = 0;
        }
    }

    pub fn begin_input_playback(&mut self, input_playing_index: usize) {
        self.input_playing_index = input_playing_index;   
        let file_name = CString::new(get_input_file_location(input_playing_index, true)).unwrap();
        unsafe { 
            self.playback_handle = Some(CreateFileA(
                file_name.as_ptr() as *const i8, 
                GENERIC_READ, 
                FILE_SHARE_READ, 
                ptr::null_mut(), 
                OPEN_EXISTING, 
                0, 
                ptr::null_mut())
            );

            if cfg!(feature = "0") {
                let mut file_position = LARGE_INTEGER::default();
                *file_position.QuadPart_mut() = get_total_allocated_memory_size() as i64;
                SetFilePointerEx(self.playback_handle.unwrap(), file_position, ptr::null_mut(), FILE_BEGIN);
            }

            let replay_buffer = self.get_replay_buffer(input_playing_index);
            RtlCopyMemory(get_allocated_memory_block(), replay_buffer.memory_block, get_total_allocated_memory_size());
        }
    }

    pub fn end_input_playback(&mut self) {
        unsafe {
            CloseHandle(self.playback_handle.unwrap());
            self.playback_handle = None;
            self.input_playing_index = 0;
        }
    }

    pub fn record_input(&mut self, new_input: *const GameInput) {
        unsafe {
            let mut bytes_written: u32 = 0;
                
            WriteFile(
                self.recording_handle.unwrap(), 
                new_input as *const c_void, 
                size_of::<GameInput>() as u32,
                 &mut bytes_written,
                  ptr::null_mut()
            );
        }
    }

    pub fn playback_input(&mut self, new_input: *mut GameInput) {
        unsafe {
            let mut bytes_read: u32 = 0;
            if ReadFile(
                self.playback_handle.unwrap(), 
                new_input as *mut c_void, 
                size_of::<GameInput>() as u32,
                &mut bytes_read,
                ptr::null_mut()
            ) > 0 {
                if bytes_read == 0 {
                    let playing_index = self.input_playing_index;
                    self.end_input_playback();
                    self.begin_input_playback(playing_index);
                    
                    ReadFile(
                        self.playback_handle.unwrap(), 
                        new_input as *mut c_void, 
                        size_of::<GameInput>() as u32,
                        &mut bytes_read,
                        ptr::null_mut()
                    );
                }
            }
        }
    }
}

fn get_input_file_location(slot_index: usize, is_input_stream: bool) -> String {
    let file_type = if is_input_stream { "input" } else { "state" };
    let temp = format!("loop_edit_{}_{}.hmi", slot_index, file_type);
    build_target_file_path_name(&temp)
}