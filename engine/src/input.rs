

#[repr(C)]
#[derive(Default, Copy, Clone)]
pub struct GameButtonState {
    pub half_tansition_count: i32,
    pub ended_down: bool,
}

pub union GameControllerButtons {
    pub all: [GameButtonState; 12],
    pub distinct: GameControllerDistinctButtons,
}

impl Default for GameControllerButtons {
    fn default() -> Self {
        Self {
            all: [GameButtonState::default(); 12]
        }
    }
}

#[repr(C)]
#[derive(Default, Copy, Clone)]
pub struct GameControllerDistinctButtons {
    pub move_up: GameButtonState,
    pub move_down: GameButtonState,
    pub move_left: GameButtonState,
    pub move_right: GameButtonState,
    pub action_up: GameButtonState,
    pub action_down: GameButtonState,
    pub action_left: GameButtonState,
    pub action_right: GameButtonState,
    pub left_shoulder: GameButtonState,
    pub right_shoulder: GameButtonState,
    pub back: GameButtonState,
    pub start: GameButtonState,
    pub terminator: GameButtonState
}

#[repr(C)]
#[derive(Default)]
pub struct GameControllerInput {
    pub is_connected: bool,
    pub is_analog: bool,
    pub stick_average_x: f32,
    pub stick_average_y: f32,
    pub buttons: GameControllerButtons,
}

pub const CONTROLLER_COUNT: usize = 5;

#[repr(C)]
#[derive(Default)]
pub struct GameInput {
    pub delta_time_for_frame: f32,
    pub mouse_buttons: [GameButtonState; CONTROLLER_COUNT],
    pub mouse_x: i32,
    pub mouse_y: i32,
    pub mouse_z: i32,
    pub controllers: [GameControllerInput; CONTROLLER_COUNT]
}

impl GameInput {
    pub fn get_controller(&mut self, controller_index: usize) -> Option<&GameControllerInput> {
        self.controllers.get(controller_index)
    }

    pub fn get_controller_mut(&mut self, controller_index: usize) -> Option<&mut GameControllerInput> {
        self.controllers.get_mut(controller_index)
    }
}