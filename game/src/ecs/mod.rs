mod entities;
mod archetypes;
mod components;
mod querying;
mod slicevec;

pub use entities::*;
pub use archetypes::*;
pub use components::*;
pub use querying::*;

#[repr(C)]
pub struct World {
    entities: Entities,
    archetypes: Archetypes,
    component_storages: ComponentStorages,
    
}

pub fn create_world() -> World {
    World {
        entities: entities(),        
        archetypes: archetypes(),
        component_storages: component_storages(),
    }
}

pub fn add_entity(world: &mut World) -> Entity {
    increment_entity_count(&mut world.entities);
    entity(last_entity_index(&world.entities))
}

pub fn add_component<C>(world: &mut World, entity: Entity, component: C)
    where C: Component {
    
    set_component_storage_if_not_set_already::<C>(&mut world.component_storages);

    if entity_is_located(&world.entities.location_map, entity) {
        let location = get_entity_location(&world.entities.location_map, entity);
        let source_archetype = get_archetype_for_entity_location(&world.archetypes, location);
        let source_archetype_index = source_archetype.index;
        let source_layout = source_archetype.layout.clone();        
        let target_layout = clone_entity_layout_and_add_component::<C>(&source_layout);
        create_archetype_if_non_existant(&mut world.archetypes, &target_layout);     
        
        let target_archetype_entity_location = move_to_next_archetype_entity_location(&mut world.archetypes, &target_layout);    
        let target_archetype = get_archetype_for_layout(&world.archetypes, &target_layout).unwrap();        
        let target_archetype_index = target_archetype.index;
        let source_archetype = get_archetype_for_entity_location(&world.archetypes, location);
            
        move_components(&mut world.component_storages, source_layout, source_archetype, location.location_in_archetype, target_archetype, target_archetype_entity_location);
        store_component_at_location(&target_archetype.chunks, target_archetype_entity_location, component);
        move_to_previous_archetype_entity_location(&mut world.archetypes, source_archetype_index); 
        change_entity_location(&mut world.entities.location_map, entity, target_archetype_index, target_archetype_entity_location);
    } else {
        let layout = create_entity_layout_from_component::<C>();
        create_archetype_if_non_existant(&mut world.archetypes, &layout);     
        let archetype_entity_location = move_to_next_archetype_entity_location(&mut world.archetypes, &layout);
        let archetype = get_or_create_archetype_mut(&mut world.archetypes, &layout);
        store_component_at_location::<C>(&archetype.chunks, archetype_entity_location, component);
        add_entity_to_location(&mut world.entities.location_map, archetype.index, archetype_entity_location);
    }
}

