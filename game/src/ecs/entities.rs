use std::marker::*;
use super::*;

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
pub struct Entity {
    index: usize,
}

pub fn entity(index: usize) -> Entity {
    Entity {
        index
    }
}

pub struct Entities {
    pub location_map: EntityLocationMap,
    entity_count: usize,
}

pub fn entities() -> Entities {
    Entities {
        location_map: entity_location_map(),
        entity_count: 0,
    }
}

pub fn increment_entity_count(entities: &mut Entities) {
    entities.entity_count += 1
}

pub fn last_entity_index(entities: &Entities) -> usize {
    entities.entity_count - 1
}

#[derive(Copy, Clone)]
pub struct EntityLocation { pub archetype_index: ArchetypeIndex, pub location_in_archetype: ArchetypeEntityLocation }

pub struct EntityLocationMap {
    inner: Vec<EntityLocation>
}

fn entity_location_map() -> EntityLocationMap {
    EntityLocationMap {
        inner: vec!()
    }
}

pub fn entity_is_located(location_map: &EntityLocationMap, entity: Entity) -> bool {
    location_map.inner.len() > entity.index
}

pub fn get_entity_location(location_map: &EntityLocationMap, entity: Entity) -> EntityLocation {
    location_map.inner[entity.index]
}

pub fn add_entity_to_location(location_map: &mut EntityLocationMap, archetype_index: ArchetypeIndex, location_in_archetype: ArchetypeEntityLocation) {
    location_map.inner.push(EntityLocation { archetype_index, location_in_archetype });
}

pub fn change_entity_location(location_map: &mut EntityLocationMap, entity: Entity, archetype_index: ArchetypeIndex, location_in_archetype: ArchetypeEntityLocation) {
    location_map.inner[entity.index] = EntityLocation { archetype_index, location_in_archetype };
}
