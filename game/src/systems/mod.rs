use crate::{ecs::*, CellPosition, CellNeighbours};

pub fn run_systems(
    world: &mut World
) {
    for (position, neighbours) in iterate_query::<CellPosition, CellNeighbours>(world) {
        println!("{:?}", position);
        println!("{:?}", neighbours);
    }
}