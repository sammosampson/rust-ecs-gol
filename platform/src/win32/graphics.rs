use std::{
    mem::*,
    ptr
};

use winapi:: {
    shared::{
        minwindef::*
    },
    um:: {
        wingdi::*, 
        winnt::*,
        memoryapi::*
    }
};

pub struct Win32OffscreenBuffer {
    pub info: BITMAPINFO,
    pub memory: *mut std::ffi::c_void,
    pub width: u32,
    pub height: u32,
    pub pitch: u32,
    pub bytes_per_pixel: u32,
}

impl Default for Win32OffscreenBuffer {
    fn default() -> Self {
        Self { 
            info: BITMAPINFO::default(),
            memory: ptr::null_mut(),
            width: 0,
            height: 0,
            pitch: 0,
            bytes_per_pixel: 0
        }
    }
}

impl Win32OffscreenBuffer {
    pub unsafe fn resize(&mut self, width: i32, height: i32) {                            
        if !self.memory.is_null() {
            VirtualFree(self.memory, 0, MEM_RELEASE);
        }
    
        self.bytes_per_pixel = 4u32;
        self.width = width as u32;
        self.height = height as u32;        
        self.pitch = self.width * self.bytes_per_pixel;
        
        self.info = BITMAPINFO::default();
        self.info.bmiHeader.biSize = size_of::<BITMAPINFOHEADER>() as UINT;
        self.info.bmiHeader.biWidth = width;
        self.info.bmiHeader.biHeight = -height;
        self.info.bmiHeader.biPlanes = 1;
        self.info.bmiHeader.biBitCount = 32;
        self.info.bmiHeader.biCompression = 0;
    
        let bitmap_memory_size = (self.width * self.height * self.bytes_per_pixel) as usize;
        self.memory = VirtualAlloc(ptr::null_mut(), bitmap_memory_size, MEM_RESERVE | MEM_COMMIT, PAGE_READWRITE);
    }    

    #[cfg(feature = "0")]
    pub fn debug_draw_vertical(&mut self, x: u32, mut top: u32, mut bottom: u32, colour: u32) {
        if top <= 0 {
            top = 0;
        }

        if bottom > self.height {
            bottom = self.height;
        }

        if x < self.width {
            unsafe {
                let mut pixel = self.memory as *mut u8;
                pixel = pixel.add((x * self.bytes_per_pixel + top * self.pitch) as usize);
                
                for _y in top..bottom {
                    *(pixel as *mut u32) = colour;
                    pixel = pixel.add(self.pitch as usize);
                }
            }
        }
    }

    #[cfg(feature = "0")]
    pub fn draw_sound_buffer_marker(
        &mut self,
        _sound_output: &Win32SoundOutput,
        c: f32,
        pad_x: u32,
        top: u32,
        bottom: u32,
        value: u32,
        colour: u32
    ) {
        let x_float = c * value as f32;
        let x = pad_x + x_float as u32;
        self.debug_draw_vertical(x, top, bottom, colour);  
    }

    #[cfg(feature = "0")]
    pub fn debug_sync_display(
        &mut self,
        marker_count: usize,
        markers: *mut Win32DebugTimeMarker,
        current_marker_index: usize,
        sound_output: &Win32SoundOutput
    ) {
        let pad_x = 16;
        let pad_y = 16;

        let line_height = 64;
        
        let c = (self.width - 2 * pad_x) as f32 / sound_output.secondary_buffer_size as f32;
        
        for marker_index in 0..marker_count {
            unsafe {
                let this_marker = &(*markers.add(marker_index));
                gol_assert!(this_marker.output_play_cursor < sound_output.secondary_buffer_size);
                gol_assert!(this_marker.output_write_cursor < sound_output.secondary_buffer_size);
                gol_assert!(this_marker.output_location < sound_output.secondary_buffer_size);
                gol_assert!(this_marker.output_byte_count < sound_output.secondary_buffer_size);
                gol_assert!(this_marker.flip_play_cursor < sound_output.secondary_buffer_size);
                gol_assert!(this_marker.flip_write_cursor < sound_output.secondary_buffer_size);
                
                let play_colour = 0xFFFFFFFF;
                let write_colour = 0xFFFF0000;
                let expected_flip_colour = 0xFFFFFF00;
                let play_window_colour = 0xFFFF00FF;

                let mut top = pad_y;
                let mut bottom = pad_y + line_height;

                if marker_index == current_marker_index {
                    top += pad_y + line_height;
                    bottom += pad_y + line_height;

                    let first_top = top;

                    self.draw_sound_buffer_marker(sound_output, c, pad_x, top, bottom, this_marker.output_play_cursor, play_colour);
                    self.draw_sound_buffer_marker(sound_output, c, pad_x, top, bottom, this_marker.output_write_cursor, write_colour);
                
                    top += pad_y + line_height;
                    bottom += pad_y + line_height;

                    self.draw_sound_buffer_marker(sound_output, c, pad_x, top, bottom, this_marker.output_location, play_colour);
                    self.draw_sound_buffer_marker(sound_output, c, pad_x, top, bottom, this_marker.output_location + this_marker.output_byte_count, write_colour);
                
                    top += pad_y + line_height;
                    bottom += pad_y + line_height;

                    self.draw_sound_buffer_marker(sound_output, c, pad_x, first_top, bottom, this_marker.flip_play_cursor, expected_flip_colour);
                }

                self.draw_sound_buffer_marker(sound_output, c, pad_x, top, bottom, this_marker.flip_play_cursor, play_colour);
                self.draw_sound_buffer_marker(sound_output, c, pad_x, top, bottom, this_marker.flip_play_cursor + 480 * sound_output.bytes_per_sample, play_window_colour);
                self.draw_sound_buffer_marker(sound_output, c, pad_x, top, bottom, this_marker.flip_write_cursor, write_colour);
            }
        }  
    }
}
