use crate::prelude::*;

use std::ptr;
use winapi::um::winuser::*;

use super::state::*;

pub fn process_xinput_digital_button(
    button_state: u16,
    old_state: &GameButtonState,
    button_bit: u16,
    new_state: &mut GameButtonState
) {
    new_state.ended_down = button_state & button_bit == button_bit;
    new_state.half_tansition_count = if old_state.ended_down != new_state.ended_down { 1 } else { 0 };
}

pub fn process_keyboard_message(new_state: &mut GameButtonState, is_down: bool) {
    if new_state.ended_down != is_down {
        new_state.ended_down = is_down;
        new_state.half_tansition_count += 1;
    }
}

pub fn process_input_stick_value(value: i16, dead_zone_threshold: i16) -> f32 {
    let mut result = 0.0;

    if value < -dead_zone_threshold {
        result = (value + dead_zone_threshold) as f32 / (32768.0 - dead_zone_threshold as f32)
    } else if value > dead_zone_threshold {
        result = (value - dead_zone_threshold) as f32 / (32767.0 - dead_zone_threshold as f32)
    }

    result
}

impl App {
    pub fn process_pending_messages(&mut self, state: &mut Win32State, keyboard_controller: &mut GameControllerInput) {
        unsafe {
            let mut message = MSG::default();

            while PeekMessageA(&mut message, ptr::null_mut(), 0, 0, PM_REMOVE) != 0 {
                match message.message {
                    WM_QUIT => {
                        self.running = false;
                    },
                    WM_SYSKEYDOWN| WM_SYSKEYUP| WM_KEYDOWN | WM_KEYUP => {
                        let vkcode = message.wParam as i32;
                        let was_down = (message.lParam & (1 << 30)) != 0;
                        let is_down = (message.lParam & (1 << 31)) == 0;
                        let alt_key_was_down = message.lParam & (1 << 29) != 0;
                        if was_down != is_down {
                            if vkcode == 'W' as i32 {
                                process_keyboard_message(&mut keyboard_controller.buttons.distinct.move_up, is_down);
                            } else if vkcode == 'A' as i32 {
                                process_keyboard_message(&mut keyboard_controller.buttons.distinct.move_left, is_down);
                            } else if vkcode == 'S' as i32 {
                                process_keyboard_message(&mut keyboard_controller.buttons.distinct.move_down, is_down);
                            } else if vkcode == 'D' as i32 {
                                process_keyboard_message(&mut keyboard_controller.buttons.distinct.move_right, is_down);
                            } else if vkcode == 'Q' as i32 {
                                process_keyboard_message(&mut keyboard_controller.buttons.distinct.left_shoulder, is_down);
                            }  else if vkcode == 'E' as i32 {
                                process_keyboard_message(&mut keyboard_controller.buttons.distinct.right_shoulder, is_down);
                            } else if vkcode == VK_UP {
                                process_keyboard_message(&mut keyboard_controller.buttons.distinct.action_up, is_down);
                            } else if vkcode == VK_LEFT {
                                process_keyboard_message(&mut keyboard_controller.buttons.distinct.action_left, is_down);
                            } else if vkcode == VK_DOWN {
                                process_keyboard_message(&mut keyboard_controller.buttons.distinct.action_down, is_down);
                            } else if vkcode == VK_RIGHT {
                                process_keyboard_message(&mut keyboard_controller.buttons.distinct.action_right, is_down);
                            } else if vkcode == VK_ESCAPE {
                                process_keyboard_message(&mut keyboard_controller.buttons.distinct.back, is_down);
                            } else if vkcode == VK_SPACE {
                                process_keyboard_message(&mut keyboard_controller.buttons.distinct.start, is_down);
                            } else if vkcode == 'P' as i32 {
                                if cfg!(feature = "gol-internal") {
                                    if is_down {
                                        self.paused = !self.paused;
                                    }
                                }
                            } else if vkcode == 'L' as i32 {
                                if cfg!(feature = "gol-internal") {
                                    if is_down {
                                        if !state.is_playing() {
                                            if !state.is_recording() {
                                                state.begin_recording_input(1);
                                            } else {
                                                state.end_recording_input();
                                                state.begin_input_playback(1);
                                            }
                                        } else {
                                            state.end_input_playback();
                                        }
                                    }
                                }
                            } 
                            if is_down {
                                if vkcode == VK_F4 && alt_key_was_down {
                                    self.running = false;
                                }

                                if vkcode == VK_RETURN && alt_key_was_down {
                                    if !message.hwnd.is_null() {
                                        self.toggle_fullscreen(message.hwnd);
                                    }
                                }
                            }
                        }
                    },
                    _ => {
                        TranslateMessage(&message);
                        DispatchMessageA(&message);
                    }
                }
            }
            
        }
    }
}