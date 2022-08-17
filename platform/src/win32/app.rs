use super::graphics::*;
use super::io::*;
use super::sound::*;
use super::input::*;
use super::state::*;
use super::windows::*;
use super::timing::*;
use super::dynamic::*;
use super::services::*;

use crate::prelude::*;

use std::{
    ffi::CString,
    ptr,
    arch::x86_64::*
};

use winapi:: {
    shared::{
        windef::*,
        minwindef::*,
        winerror::*
    },
    um:: {
        timeapi::*,
        synchapi::*,
        mmsystem::*,
        winuser::*,
        winnt::*,
        libloaderapi::*,
        dsound::*,
        profileapi::*,
        memoryapi::*,
        xinput::*,
        wingdi::*
    }
};

pub static mut GLOBAL_APPLICATION: Option<App> = None;

pub struct App {
    instance: HINSTANCE,
    pub window_placement: WINDOWPLACEMENT,
    pub running: bool,
    pub debug_show_cursor: bool,
    pub paused: bool,
    pub buffer: Win32OffscreenBuffer,
    pub sound_buffer: LPDIRECTSOUNDBUFFER,
    pub perf_count_frequency: i64
}

impl App {
    pub fn start() {
        unsafe {
            let app = Self {
                running: false,
                debug_show_cursor: cfg!(feature = "gol-internal"),
                paused: false,
                buffer: Win32OffscreenBuffer::default(),
                sound_buffer: ptr::null_mut(),
                instance: GetModuleHandleA(ptr::null_mut()),
                window_placement: WINDOWPLACEMENT::default(),
                perf_count_frequency: 0
            };

            GLOBAL_APPLICATION = Some(app);
            
            if let Some(app) = &mut GLOBAL_APPLICATION {
                app.run();
            }
        }
    }

    fn run(&mut self) {
        let class_name = CString::new("HandmadeHeroWindowClass").unwrap();
        let window_name = CString::new("Handmade Hero").unwrap();

        unsafe {
            let mut perf_count_frequency_result = LARGE_INTEGER::default();
            QueryPerformanceFrequency(&mut perf_count_frequency_result);
            self.perf_count_frequency = *perf_count_frequency_result.QuadPart();
            
            let desired_scheduler_ms = 1;
            let sleep_is_granular = timeBeginPeriod(desired_scheduler_ms) == TIMERR_NOERROR;
            
            let mut window_class = WNDCLASSA::default();   
            
            self.buffer.resize(960, 540);
            
            window_class.style = CS_HREDRAW|CS_VREDRAW;
            window_class.lpfnWndProc = Some(main_window_callback);
            window_class.lpszClassName = class_name.as_ptr() as *const i8;
            window_class.hInstance = self.instance;
            window_class.hCursor = LoadCursorW(ptr::null_mut(), IDC_ARROW);

            if RegisterClassA(&window_class) > 0 {
                let window = CreateWindowExA(
                    0, //WS_EX_TOPMOST|WS_EX_LAYERED,
                    window_class.lpszClassName,
                    window_name.as_ptr() as *const i8,
                    WS_OVERLAPPEDWINDOW|WS_VISIBLE,
                    CW_USEDEFAULT,
                    CW_USEDEFAULT,
                    CW_USEDEFAULT,
                    CW_USEDEFAULT,
                    ptr::null_mut(),
                    ptr::null_mut(),
                    self.instance,
                    ptr::null_mut());
                
                if !window.is_null() {
                    let mut monitor_refresh_hz = 60;
                    let refresh_dc = GetDC(window);
                    let refresh_rate = GetDeviceCaps(refresh_dc, VREFRESH);
                    if refresh_rate > 1 {
                       monitor_refresh_hz = refresh_rate;
                    }
                    let game_update_hz = monitor_refresh_hz as f32 / 2.0;
                    let target_seconds_per_frame = 1.0 / game_update_hz;
        
                    let mut sound_output = Win32SoundOutput::new(game_update_hz);
                    self.initialise_sound(window, sound_output.samples_per_second, sound_output.secondary_buffer_size);
                    self.clear_sound_buffer(&mut sound_output);
                    self.sound_buffer.as_ref().unwrap().Play(0, 0, DSBPLAY_LOOPING);

                    self.running = true;

                    if cfg!(feature = "0") {
                        while self.running {
                            let mut play_cursor = 0;
                            let mut write_cursor = 0;

                            self.sound_buffer.as_ref().unwrap().GetCurrentPosition(
                                &mut play_cursor, 
                                &mut write_cursor);

                            println!("PC:{:?} WC:{:?}", play_cursor, write_cursor);
                        }
                    }

                    let samples = VirtualAlloc(
                        ptr::null_mut(), 
                        sound_output.secondary_buffer_size as usize, 
                        MEM_RESERVE | MEM_COMMIT, 
                        PAGE_READWRITE
                    );
    
                    let mut game_memory = GameMemory {
                        is_initialized: false,
                        root: ptr::null_mut(),
                        debug_platform_read_entire_file_func: Box::new(debug_platform_read_entire_file), 
                        debug_platform_write_entire_file_func: Box::new(debug_platform_write_entire_file), 
                        debug_platform_free_file_memory_func: Box::new(debug_platform_free_file_memory),
                    };
                                        
                    let mut state = Win32State::new();
                
                
                    if !samples.is_null() {
                        let mut new_input = &mut GameInput::default();
                        let mut old_input = &mut GameInput::default();
                        
                        let mut last_counter = get_wall_clock();
                        let mut flip_wall_clock = get_wall_clock();

                        let mut debug_time_marker_index = 0;
                        let mut debug_time_markers = [Win32DebugTimeMarker::default(); 30];

                        let mut audio_latency_bytes;
                        let mut audio_latency_seconds;
                        let mut sound_is_valid = false;

                        let game_dll_name = build_target_file_path_name("gol_game.dll");
                        let temp_game_dll_name = build_target_file_path_name("gol_game_temp.dll");

                        let mut game_code = Win32GameCode::new(game_dll_name, temp_game_dll_name);
                        game_code.load();
                        
                        let mut last_cycle_count = _rdtsc();

                        while self.running
                        {
                            new_input.delta_time_for_frame = target_seconds_per_frame;
                            
                            game_code.reload_if_source_dll_is_newer();

                            if let Some(new_keyboard_controller) = new_input.get_controller_mut(0) {
                                new_keyboard_controller.is_connected = true;
                                if let Some(old_keyboard_controller) = old_input.get_controller(0) {
                                    for button_index in 0..new_keyboard_controller.buttons.all.len() {
                                        new_keyboard_controller.buttons.all[button_index] = old_keyboard_controller.buttons.all[button_index]; 
                                    }
                                }
                                self.process_pending_messages(&mut state, new_keyboard_controller);
                            }

                            let mut controller_state = XINPUT_STATE::default();

                            if !self.paused {
                                let mut mouse_p = POINT::default();
                                GetCursorPos(&mut mouse_p as LPPOINT);
                                ScreenToClient(window, &mut mouse_p as LPPOINT);

                                new_input.mouse_x = mouse_p.x;
                                new_input.mouse_y = mouse_p.y;
                                new_input.mouse_z = 0;
                                process_keyboard_message(&mut new_input.mouse_buttons[0], GetKeyState(VK_LBUTTON) & (1 << 15) != 0);
                                process_keyboard_message(&mut new_input.mouse_buttons[1], GetKeyState(VK_MBUTTON) & (1 << 15) != 0);
                                process_keyboard_message(&mut new_input.mouse_buttons[2], GetKeyState(VK_RBUTTON) & (1 << 15) != 0);
                                process_keyboard_message(&mut new_input.mouse_buttons[3], GetKeyState(VK_XBUTTON1) & (1 << 15) != 0);
                                process_keyboard_message(&mut new_input.mouse_buttons[4], GetKeyState(VK_XBUTTON2) & (1 << 15) != 0);


                                let mut max_controller_count = XUSER_MAX_COUNT;
                                if max_controller_count > new_input.controllers.len() as u32 - 1 {
                                    max_controller_count = new_input.controllers.len() as u32 - 1
                                }

                                for controller_index in 0..max_controller_count {
                                    let our_controller_index = controller_index as usize + 1;
                                    if let Some(old_controller) = old_input.get_controller(our_controller_index) {
                                        if let Some(new_controller) = new_input.get_controller_mut(our_controller_index) {
                                    
                                            if XInputGetState(controller_index, &mut controller_state) == ERROR_SUCCESS {
                                                new_controller.is_connected = true;
                                                new_controller.is_analog = old_controller.is_analog;

                                                let pad = &controller_state.Gamepad;

                                                new_controller.stick_average_x = process_input_stick_value(pad.sThumbLX, XINPUT_GAMEPAD_LEFT_THUMB_DEADZONE);
                                                new_controller.stick_average_y = process_input_stick_value(pad.sThumbLY, XINPUT_GAMEPAD_LEFT_THUMB_DEADZONE);
                                                
                                                if new_controller.stick_average_x != 0.0 || new_controller.stick_average_y != 0.0 {
                                                    new_controller.is_analog = true;
                                                }

                                                if pad.wButtons & XINPUT_GAMEPAD_DPAD_UP == XINPUT_GAMEPAD_DPAD_UP {
                                                    new_controller.stick_average_y = 1.0;
                                                    new_controller.is_analog = false;
                                                }
                                                if pad.wButtons & XINPUT_GAMEPAD_DPAD_DOWN == XINPUT_GAMEPAD_DPAD_DOWN {
                                                    new_controller.stick_average_y = -1.0;
                                                    new_controller.is_analog = false;
                                                }
                                                if pad.wButtons & XINPUT_GAMEPAD_DPAD_LEFT == XINPUT_GAMEPAD_DPAD_LEFT {
                                                    new_controller.stick_average_x = -1.0;
                                                    new_controller.is_analog = false;
                                                }
                                                if pad.wButtons & XINPUT_GAMEPAD_DPAD_RIGHT == XINPUT_GAMEPAD_DPAD_RIGHT {
                                                    new_controller.stick_average_x = 1.0;
                                                    new_controller.is_analog = false;
                                                }

                                                let threshold = 0.5;
                                                
                                                process_xinput_digital_button(
                                                    if new_controller.stick_average_x < -threshold { 1 } else { 0 },
                                                    &old_controller.buttons.distinct.move_left, 
                                                    1, 
                                                    &mut new_controller.buttons.distinct.move_left
                                                );
                                                process_xinput_digital_button(
                                                    if new_controller.stick_average_x > threshold { 1 } else { 0 },
                                                    &old_controller.buttons.distinct.move_right, 
                                                    1, 
                                                    &mut new_controller.buttons.distinct.move_right
                                                );
                                                process_xinput_digital_button(
                                                    if new_controller.stick_average_y < -threshold { 1 } else { 0 },
                                                    &old_controller.buttons.distinct.move_down, 
                                                    1, 
                                                    &mut new_controller.buttons.distinct.move_down
                                                );
                                                process_xinput_digital_button(
                                                    if new_controller.stick_average_y > threshold { 1 } else { 0 },
                                                    &old_controller.buttons.distinct.move_up, 
                                                    1, 
                                                    &mut new_controller.buttons.distinct.move_up
                                                );
                                                process_xinput_digital_button(
                                                    pad.wButtons, 
                                                    &old_controller.buttons.distinct.action_left, 
                                                    XINPUT_GAMEPAD_X, 
                                                    &mut new_controller.buttons.distinct.action_left
                                                );
                                                process_xinput_digital_button(
                                                    pad.wButtons, 
                                                    &old_controller.buttons.distinct.action_right, 
                                                    XINPUT_GAMEPAD_B, 
                                                    &mut new_controller.buttons.distinct.action_right
                                                );
                                                process_xinput_digital_button(
                                                    pad.wButtons, 
                                                    &old_controller.buttons.distinct.action_down, 
                                                    XINPUT_GAMEPAD_A, 
                                                    &mut new_controller.buttons.distinct.action_down
                                                );
                                                process_xinput_digital_button(
                                                    pad.wButtons, 
                                                    &old_controller.buttons.distinct.action_up, 
                                                    XINPUT_GAMEPAD_Y, 
                                                    &mut new_controller.buttons.distinct.action_up
                                                );
                                                process_xinput_digital_button(
                                                    pad.wButtons, 
                                                    &old_controller.buttons.distinct.left_shoulder, 
                                                    XINPUT_GAMEPAD_LEFT_SHOULDER, 
                                                    &mut new_controller.buttons.distinct.left_shoulder
                                                );
                                                process_xinput_digital_button(
                                                    pad.wButtons, 
                                                    &old_controller.buttons.distinct.right_shoulder, 
                                                    XINPUT_GAMEPAD_RIGHT_SHOULDER, 
                                                    &mut new_controller.buttons.distinct.right_shoulder
                                                );
                                                process_xinput_digital_button(
                                                    pad.wButtons, 
                                                    &old_controller.buttons.distinct.start, 
                                                    XINPUT_GAMEPAD_START, 
                                                    &mut new_controller.buttons.distinct.start
                                                );    
                                                process_xinput_digital_button(
                                                    pad.wButtons, 
                                                    &old_controller.buttons.distinct.back, 
                                                    XINPUT_GAMEPAD_BACK, 
                                                    &mut new_controller.buttons.distinct.back
                                                );                                           
                                            }
                                            else {
                                                new_controller.is_connected = false;
                                            }
                                        }
                                    }
                                }

                                let mut thread_context = ThreadContext::default();
                                
                                let mut screen_buffer = GameOffscreenBuffer { 
                                    memory: self.buffer.memory,
                                    width: self.buffer.width,
                                    height: self.buffer.height,
                                    pitch: self.buffer.pitch,
                                    bytes_per_pixel: self.buffer.bytes_per_pixel
                                };

                                if state.is_recording() {
                                    state.record_input(new_input);
                                }
                                
                                if state.is_playing() {
                                    state.playback_input(new_input);
                                }

                                game_code.game_update_and_render(&mut thread_context, &mut game_memory, &mut new_input, &mut screen_buffer);

                                let audio_wall_clock = get_wall_clock();
                                let from_begin_to_audio_seconds = self.get_seconds_elapsed(flip_wall_clock, audio_wall_clock);

                                let mut play_cursor = 0;
                                let mut write_cursor = 0;

                                if self
                                    .sound_buffer
                                    .as_ref()
                                    .unwrap()
                                    .GetCurrentPosition(&mut play_cursor, &mut write_cursor) == DS_OK {
                                    
                                    if !sound_is_valid {
                                        sound_output.running_sample_index = write_cursor / sound_output.bytes_per_sample;
                                        sound_is_valid = true;
                                    }

                                    let byte_to_lock = (
                                        sound_output.running_sample_index * sound_output.bytes_per_sample
                                        ) % sound_output.secondary_buffer_size;
                                    
                                    let expected_sound_bytes_per_frame = 
                                        (((sound_output.samples_per_second * sound_output.bytes_per_sample) as f32 / game_update_hz) as f32) as u32;

                                    let seconds_until_flip = target_seconds_per_frame - from_begin_to_audio_seconds;
                                    let expected_bytes_until_flip = (seconds_until_flip / target_seconds_per_frame) * expected_sound_bytes_per_frame as f32;
                                    let expected_frame_boundary_byte = play_cursor + expected_bytes_until_flip as u32;
                                    
                                    let mut safe_write_cursor = write_cursor;
                                    if safe_write_cursor < play_cursor {
                                        safe_write_cursor += sound_output.secondary_buffer_size
                                    }
                                    
                                    gol_assert!(safe_write_cursor >= play_cursor);
                                    safe_write_cursor += sound_output.safety_bytes;

                                    let audio_card_is_low_latency = safe_write_cursor < expected_frame_boundary_byte;
                                    
                                    let mut target_cursor;
                                    if audio_card_is_low_latency {   
                                        target_cursor = expected_frame_boundary_byte + expected_sound_bytes_per_frame;
                                    } else {
                                        target_cursor = write_cursor + expected_sound_bytes_per_frame + sound_output.safety_bytes;
                                    }

                                    target_cursor = target_cursor % sound_output.secondary_buffer_size;
                  
                                    let mut bytes_to_write;
                                    if byte_to_lock > target_cursor {    
                                        bytes_to_write = sound_output.secondary_buffer_size - byte_to_lock;
                                        bytes_to_write += target_cursor;
                                    } else {
                                        bytes_to_write = target_cursor - byte_to_lock;
                                    }

                                    let mut sound_buffer = GameSoundOutputBuffer {
                                        samples_per_second: sound_output.samples_per_second,
                                        sample_count: bytes_to_write / sound_output.bytes_per_sample,
                                        samples,
                                    };
                                    
                                    game_code.get_game_sound_samples(&mut thread_context, &mut game_memory, &mut sound_buffer);
                                
                                    if cfg!(feature = "gol-internal") {
                                        let mut marker = &mut debug_time_markers[debug_time_marker_index];
                                        marker.output_play_cursor = play_cursor;
                                        marker.output_write_cursor = write_cursor;
                                        marker.output_location = byte_to_lock;
                                        marker.output_byte_count = bytes_to_write;
                                        marker.expected_flip_play_cursor = expected_frame_boundary_byte;
                                    
                                        let mut unwrapped_write_cursor = write_cursor;
                                        if unwrapped_write_cursor < play_cursor {
                                            unwrapped_write_cursor += sound_output.secondary_buffer_size;
                                        }

                                        audio_latency_bytes = unwrapped_write_cursor - play_cursor;
                                        audio_latency_seconds = 
                                            (audio_latency_bytes as f32 / sound_output.bytes_per_sample as f32) /
                                            sound_output.samples_per_second as f32;
                                                      
                                        if cfg!(feature = "0") {
                                            println!(
                                                "BTL:{:?} TC:{:?} BTW:{:?} - PC:{:?} WC:{:?} Delta:{:?} ({:?}s)", 
                                                byte_to_lock,
                                                target_cursor,
                                                bytes_to_write,
                                                play_cursor,
                                                write_cursor,
                                                audio_latency_bytes,
                                                audio_latency_seconds
                                            );
                                        }
                                    }

                                    self.fill_sound_buffer(&mut sound_output, byte_to_lock, bytes_to_write, &sound_buffer);
                                } else {
                                    sound_is_valid = false;
                                }
                        
                                let work_counter = get_wall_clock();
                                let work_seconds_elapsed = self.get_seconds_elapsed(last_counter, work_counter);
                                
                                let mut seconds_elapsed_for_frame = work_seconds_elapsed;
                                if seconds_elapsed_for_frame < target_seconds_per_frame {                        
                                    if sleep_is_granular {
                                        let sleep_ms = (1000.0 * (target_seconds_per_frame - seconds_elapsed_for_frame)) as u32;
                                        if sleep_ms > 0 {
                                            Sleep(sleep_ms);
                                        }
                                    }
                                    
                                    let test_seconds_elapsed_for_frame = self.get_seconds_elapsed(last_counter, get_wall_clock());
                                    
                                    if test_seconds_elapsed_for_frame < target_seconds_per_frame {
                                    }
                                    
                                    while seconds_elapsed_for_frame < target_seconds_per_frame {                            
                                        seconds_elapsed_for_frame = self.get_seconds_elapsed(last_counter, get_wall_clock());
                                    }
                                } else {
                                }  
                                
                                let end_counter = get_wall_clock();
                                let ms_per_frame = 1000.0 * self.get_seconds_elapsed(last_counter, end_counter);
                                last_counter = end_counter;
                                
                                let dimension = Win32WindowDimension::from(window);

                                let device_context = GetDC(window);
                                self.update_window(device_context, dimension.width, dimension.height);
                                ReleaseDC(window, device_context);
                                
                                flip_wall_clock = get_wall_clock();

                                if cfg!(feature = "gol-internal") {
                                    let mut play_cursor = 0;
                                    let mut write_cursor = 0;

                                    if self
                                        .sound_buffer
                                        .as_ref()
                                        .unwrap()
                                        .GetCurrentPosition(&mut play_cursor,&mut write_cursor
                                    ) == DS_OK {
                                        gol_assert!(debug_time_marker_index < debug_time_markers.len());
                                        let mut marker = &mut debug_time_markers[debug_time_marker_index];
                                        marker.flip_play_cursor = play_cursor;
                                        marker.flip_write_cursor = write_cursor;
                                    
                                    }
                                }
                                
                                let temp = new_input;
                                new_input = old_input;
                                old_input = temp;
                                                               
                                if cfg!(feature = "0") {
                                    let end_cycle_count = _rdtsc();
                                    let cycles_elapsed = end_cycle_count - last_cycle_count;
                                    last_cycle_count = end_cycle_count;

                                    let fps = 0.0;
                                    let mcpf = cycles_elapsed as f64 / (1000.0 * 1000.0);
                            
                                    println!("{:?}fms/f,  {:?}ff/s,  {:?}fmc/f", ms_per_frame, fps, mcpf);
                                }

                                if cfg!(feature = "gol-internal") {
                                    let debug_time_marker_count = debug_time_markers.len();
                                    debug_time_marker_index += 1;
                                    if debug_time_marker_index == debug_time_marker_count {
                                        debug_time_marker_index = 0;
                                    }
                                }
                            }
                        }
                    } else {
                    }
                } else {
                }
            } else {
            }
        }
    }
}