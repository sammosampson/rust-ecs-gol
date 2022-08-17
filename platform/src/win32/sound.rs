use std::{
    mem::*,
    ptr
};

use winapi:: {
    shared::{
        windef::*,
        minwindef::*,
        winerror::*, 
        mmreg::*,
    },
    um::dsound::*
};

use crate::prelude::*;

use super::*;

#[derive(Default, Copy, Clone)]
pub struct Win32DebugTimeMarker {
    pub output_play_cursor: u32,
    pub output_write_cursor: u32,
    pub output_location: u32,
    pub output_byte_count: u32,
    pub expected_flip_play_cursor: u32,
    pub flip_play_cursor: u32,
    pub flip_write_cursor: u32,
}

pub struct Win32SoundOutput {
    pub samples_per_second:u32,
    pub running_sample_index: u32,
    pub bytes_per_sample: u32,
    pub secondary_buffer_size: u32,     
    pub safety_bytes: u32
}

impl Win32SoundOutput {
    pub fn new(game_update_hz: f32) -> Self {
        let samples_per_second = 48000;
        let bytes_per_sample = size_of::<i16>() as u32 * 2;
                
        Self {
            samples_per_second,
            running_sample_index: 0,
            bytes_per_sample,
            secondary_buffer_size: samples_per_second * bytes_per_sample,
            safety_bytes: ((samples_per_second as f32 * bytes_per_sample as f32 / game_update_hz) / 3.0) as u32
        }
    }
}

impl App {
    pub unsafe fn initialise_sound(&mut self, window: HWND, samples_per_second: u32, buffer_size: u32) {
        let mut direct_sound: LPDIRECTSOUND = ptr::null_mut();
    
        if SUCCEEDED(DirectSoundCreate(ptr::null_mut(), &mut direct_sound, ptr::null_mut())) {
            let mut wave_format = WAVEFORMATEX::default();
            wave_format.wFormatTag = WAVE_FORMAT_PCM;
            wave_format.nChannels = 2;
            wave_format.nSamplesPerSec = samples_per_second;
            wave_format.wBitsPerSample = 16;
            wave_format.nBlockAlign = (wave_format.nChannels * wave_format.wBitsPerSample) / 8;
            wave_format.nAvgBytesPerSec = wave_format.nSamplesPerSec * wave_format.nBlockAlign as u32;
            wave_format.cbSize = 0;
            if SUCCEEDED(direct_sound.as_ref().unwrap().SetCooperativeLevel(window, DSSCL_PRIORITY)) {
                let mut buffer_description = DSBUFFERDESC::default();
                buffer_description.dwSize = size_of::<DSBUFFERDESC>() as u32;
                buffer_description.dwFlags = DSBCAPS_PRIMARYBUFFER;
                let mut primary_buffer: LPDIRECTSOUNDBUFFER = ptr::null_mut();

                if SUCCEEDED(direct_sound.as_ref().unwrap().CreateSoundBuffer(&buffer_description, &mut primary_buffer, ptr::null_mut())) {
                    let error = primary_buffer.as_ref().unwrap().SetFormat(&wave_format);
                    if SUCCEEDED(error) {
                        println!("Primary buffer format was set.");
                    } else {
                    }
                } else {
                }
            } else {
            }

            let mut buffer_description = DSBUFFERDESC::default();
            buffer_description.dwSize = size_of::<DSBUFFERDESC>() as u32;
            buffer_description.dwFlags = DSBCAPS_GETCURRENTPOSITION2;
            buffer_description.dwBufferBytes = buffer_size;
            buffer_description.lpwfxFormat = &mut wave_format;

            let error = direct_sound.as_ref().unwrap().CreateSoundBuffer(&buffer_description, &mut self.sound_buffer, ptr::null_mut());
            if SUCCEEDED(error) {
                println!("Secondary buffer format was set.");
            }
        } else {
        }
    }

    pub unsafe fn fill_sound_buffer(&mut self, sound_output: &mut Win32SoundOutput, byte_to_lock: u32, bytes_to_write: u32, sound_buffer: &GameSoundOutputBuffer) {
        let mut region_1: LPVOID = ptr::null_mut();
        let mut region_2: LPVOID = ptr::null_mut();
        let mut region_1_size = 0;
        let mut region_2_size = 0;

        if SUCCEEDED(self.sound_buffer.as_ref().unwrap().Lock(
            byte_to_lock, 
            bytes_to_write, 
            &mut region_1, 
            &mut region_1_size, 
            &mut region_2, 
            &mut region_2_size, 
            0)
        ) {
            let region_1_sample_count = region_1_size / sound_output.bytes_per_sample;
            let mut dest_sample = region_1 as *mut i16;
            let mut source_sample = sound_buffer.samples as *mut i16;

            for _sample_index in 0..region_1_sample_count {
                *dest_sample = *source_sample;
                dest_sample = dest_sample.add(1);
                source_sample = source_sample.add(1);

                *dest_sample = *source_sample;
                dest_sample = dest_sample.add(1);
                source_sample = source_sample.add(1);

                sound_output.running_sample_index += 1;                
            }

            let region_2_sample_count = region_2_size / sound_output.bytes_per_sample;
            let mut dest_sample = region_2 as *mut i16;
            let mut source_sample = sound_buffer.samples as *mut i16;

            for _sample_index in 0..region_2_sample_count {
                *dest_sample = *source_sample;
                dest_sample = dest_sample.add(1);
                source_sample = source_sample.add(1);
                
                *dest_sample = *source_sample;
                dest_sample = dest_sample.add(1);
                source_sample = source_sample.add(1);

                sound_output.running_sample_index += 1;
            }

            self.sound_buffer.as_ref().unwrap().Unlock(region_1, region_1_size, region_2, region_2_size);
        }
    }

    pub unsafe fn clear_sound_buffer(&mut self, sound_output: &mut Win32SoundOutput) {
        let mut region_1 = ptr::null_mut();
        let mut region_2 = ptr::null_mut();
        let mut region_1_size = 0;
        let mut region_2_size = 0;

        if SUCCEEDED(self.sound_buffer.as_ref().unwrap().Lock(
            0, 
            sound_output.secondary_buffer_size, 
            &mut region_1, 
            &mut region_1_size, 
            &mut region_2, 
            &mut region_2_size, 
            0)
        ) {
            let mut dest_sample = region_1 as *mut u8;

            for _byte_index in 0..region_1_size {
                *dest_sample = 0;
                dest_sample = dest_sample.add(1);
            }

            let mut dest_sample = region_2 as *mut u8;

            for _byte_index in 0..region_2_size {
                *dest_sample = 0;
                dest_sample = dest_sample.add(1);
            }

            self.sound_buffer.as_ref().unwrap().Unlock(region_1, region_1_size, region_2, region_2_size);
        }
    }
}
