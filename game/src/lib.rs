mod math;
mod ecs;
mod systems;

use gol_engine::*;
use ecs::*;
use math::*;
use systems::run_systems;

#[no_mangle]
pub extern "C" fn game_update_and_render(
    _thread_context: &mut ThreadContext,
    game_memory: &mut GameMemory, 
    _game_input: &mut GameInput, 
    _buffer: &mut GameOffscreenBuffer
) {
    if !initialised(game_memory) {    
        let mut world = Box::new(create_world());
        set_game_memory_root(game_memory, world.as_mut());
        add_initial_entities_to_world(get_game_memory_root(game_memory));
        mark_as_initialised(game_memory);
    }
    
    run_systems(get_game_memory_root(game_memory));

}

#[no_mangle]
pub extern "C" fn get_game_sound_samples(
    _thread_context: &mut ThreadContext,
    _game_memory: &mut GameMemory, 
    _sound_buffer: &mut GameSoundOutputBuffer
) {
}

fn add_initial_entities_to_world(world: &mut World) {
    let cell_1 = add_entity(world);
    let cell_2 = add_entity(world);
    let cell_3 = add_entity(world);
    add_component(world, cell_1, CellNeighbours { north: None, east: Some(cell_2), south: Some(cell_3), west: None });
    add_component(world, cell_1, CellPosition(v2(0.0, 0.0)));
    
    add_component(world, cell_2, CellNeighbours{ north: None, east: None, south: None, west: Some(cell_1) });
    add_component(world, cell_2, CellPosition(v2(1.0, 0.0)));

    add_component(world, cell_3, CellNeighbours{ north: Some(cell_1), east: None, south: None, west: None });
    add_component(world, cell_3, CellPosition(v2(0.0, 1.0)));
}

#[derive(Debug)]
pub struct CellPosition(V2);

#[derive(Debug)]
pub struct CellNeighbours {
    pub north: Option<Entity>,
    pub east: Option<Entity>,
    pub south: Option<Entity>,
    pub west: Option<Entity>,
}