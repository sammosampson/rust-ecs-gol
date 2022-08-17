use crate::prelude::*;

use std::ffi::*;

use winapi::{
    um::{
        libloaderapi::*,
        winbase::*,
        fileapi::*,
        minwinbase::*, 
        synchapi::*
    }, 
    shared::minwindef::*
};
            
pub struct Win32GameCode {
    source_dll_name: String,
    temp_dll_name: String,
    dll: Option<HINSTANCE>,
    dll_last_write_time: FILETIME,
    game_update_and_render_func: Option<Box<GameUpdateAndRender>>, 
    get_game_sound_samples_func: Option<Box<GetGameSoundSamples>>
}

type GameUpdateAndRender = fn(&mut ThreadContext, &mut GameMemory, &mut GameInput, &mut GameOffscreenBuffer);
type GetGameSoundSamples = fn(&mut ThreadContext, &mut GameMemory, &mut GameSoundOutputBuffer);

pub fn get_last_write_time(file_name: &str) -> FILETIME {
    let file_name = CString::new(file_name).unwrap();
        
    let mut last_write_time = FILETIME::default();
    let mut data = WIN32_FILE_ATTRIBUTE_DATA::default();
    let data_pointer = &mut data as *mut WIN32_FILE_ATTRIBUTE_DATA;
        
    unsafe {
        if GetFileAttributesExA(file_name.as_ptr() as *const i8, GetFileExInfoStandard, data_pointer as *mut c_void) > 0 {
            last_write_time = data.ftLastWriteTime;
        }
    }
    
    last_write_time
}

impl Win32GameCode {
    pub fn new(source_dll_name: String, temp_dll_name: String) -> Self {
        Self {
            source_dll_name,
            temp_dll_name,
            dll: None,
            dll_last_write_time: FILETIME::default(),
            game_update_and_render_func: None,
            get_game_sound_samples_func: None
        }
    }

    pub fn load(&mut self) {
        let source_dll_name = CString::new(&*self.source_dll_name).unwrap();
        let temp_dll_name = CString::new(&*self.temp_dll_name).unwrap();
        
        unsafe {
            loop {
                if CopyFileA(
                    source_dll_name.as_ptr() as *const i8, 
                    temp_dll_name.as_ptr() as *const i8, 
                    FALSE
                ) == 0 {
                    println!("cannot copy game dll - retrying");
                    Sleep(1000)
                } else {
                    break;
                }
            }

            let dll = LoadLibraryA(temp_dll_name.as_ptr() as *const i8);
            
            if !dll.is_null() {
                self.dll = Some(dll);
                self.dll_last_write_time = get_last_write_time(&self.source_dll_name);
                self.game_update_and_render_func = get_game_update_and_render_function_from_library(dll);
                self.get_game_sound_samples_func = get_get_game_sound_samples_function_from_library(dll);
            } else {
                println!("cannot load game dll");
            }
        }
    }

    pub fn reload_if_source_dll_is_newer(&mut self) {
        let new_dll_last_write_time = get_last_write_time(&self.source_dll_name);
        unsafe {
            if CompareFileTime(&new_dll_last_write_time, &self.dll_last_write_time) != 0 {
                self.unload();
                self.load();    
            }
        }
    }

    fn unload(&mut self) {
        if let Some(dll) = self.dll {
            unsafe {
                FreeLibrary(dll);

            } 
            
            self.dll = None;
            self.game_update_and_render_func = None;
            self.get_game_sound_samples_func = None;
        }
    }

    pub fn game_update_and_render(
        &self,
        thread_context: &mut ThreadContext, 
        game_memory: &mut GameMemory, 
        game_input: &mut GameInput, 
        buffer: &mut GameOffscreenBuffer
    ) {
        if let Some(game_update_and_render_func) = &self.game_update_and_render_func {
            (game_update_and_render_func)(thread_context, game_memory, game_input, buffer);
        }
    }
    
    pub fn get_game_sound_samples(
        &self,        
        thread_context: &mut ThreadContext, 
        game_memory: &mut GameMemory, 
        sound_buffer: &mut GameSoundOutputBuffer
    
    ) {
        if let Some(get_game_sound_samples_func) = &self.get_game_sound_samples_func {
            (get_game_sound_samples_func)(thread_context, game_memory, sound_buffer);
        }
    }    
}

fn get_game_update_and_render_function_from_library(dll: HINSTANCE) -> Option<Box::<GameUpdateAndRender>> {
    unsafe {        
        let fn_name = CString::new("game_update_and_render").unwrap();
        let fn_pointer = GetProcAddress(dll, fn_name.as_ptr() as *const i8);
        if fn_pointer.is_null() {
            return None;
        }
        Some(Box::new(std::mem::transmute::<*const (), GameUpdateAndRender>(fn_pointer as *const())))
    }
}

fn get_get_game_sound_samples_function_from_library(dll: HINSTANCE) -> Option<Box::<GetGameSoundSamples>> {
    unsafe {        
        let fn_name = CString::new("get_game_sound_samples").unwrap();
        let fn_pointer = GetProcAddress(dll, fn_name.as_ptr() as *const i8);
        if fn_pointer.is_null() {
            return None;
        }
        Some(Box::new(std::mem::transmute::<*const (), GetGameSoundSamples>(fn_pointer as *const())))
    }
}