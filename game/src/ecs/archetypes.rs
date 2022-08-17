
use std::{
    iter::*,
    marker::*
};

use super::{*, slicevec::SliceVec};

#[derive(Copy, Clone, Default)]
pub struct ArchetypeIndex(usize);

impl From<ArchetypeIndex> for usize {
    fn from(from: ArchetypeIndex) -> Self {
        from.0
    }
}

#[derive(Copy, Clone)]
pub struct ArchetypeEntityLocation { pub chunk: ChunkIndex, pub component: ComponentIndex }

fn create_archetype_entity_location(chunk: ChunkIndex, component: ComponentIndex) -> ArchetypeEntityLocation {
    ArchetypeEntityLocation { chunk, component }
}

pub struct ArchetypeComponentSearchIndex {
    component_slices: SliceVec<ComponentTypeId>
}

fn archetype_component_search_index() -> ArchetypeComponentSearchIndex {
    ArchetypeComponentSearchIndex { 
        component_slices: SliceVec::default()
    }
}

fn push_layout_to_search(search_index: &mut ArchetypeComponentSearchIndex, layout: &EntityLayout) {
    search_index.component_slices.push(copy_layout_components(layout))
}

pub fn search_archetypes_for<'a, F: LayoutFilter + 'a>(
    search_index: &'a ArchetypeComponentSearchIndex,
    filter: F,
    from_position: usize
) -> impl Iterator<Item = ArchetypeIndex> + 'a {
    search_index.component_slices
        .iter_from(from_position)
        .enumerate()
        .filter(move |(_, components)| filter.matches_layout(components).is_pass())
        .map(move |(i, _)| ArchetypeIndex(i + from_position))
}

pub struct Archetypes {
    inner: Vec<Archetype>,
    chunk_pool: ComponentChunkPool,
    pub search_index: ArchetypeComponentSearchIndex
}


pub fn archetypes() -> Archetypes {
    Archetypes {
        inner: Vec::<Archetype>::default(),
        chunk_pool: ComponentChunkPool::default(),
        search_index: archetype_component_search_index()
    }
}

pub fn get_or_create_archetype_mut<'a>(archetypes: &'a mut Archetypes, layout: &EntityLayout) -> &'a mut Archetype {
    create_archetype_if_non_existant(archetypes, layout);    
    get_archetype_for_layout_mut(archetypes, &layout).unwrap()
}

pub fn create_archetype_if_non_existant(archetypes: &mut Archetypes, layout: &EntityLayout) {
    if !contains_archetype(archetypes, &layout) {
        let archetype = create_archetype(layout.clone(), get_next_index(archetypes));
        push_layout_to_search(&mut archetypes.search_index, &archetype.layout);
        append_archetype(archetypes, archetype);
    }
}

fn contains_archetype(archetypes: &Archetypes, layout: &EntityLayout) -> bool {
    search_archetypes_for(&archetypes.search_index, layout.clone(), 0).next().is_some()
}

fn get_next_index(archetypes: &Archetypes) -> ArchetypeIndex {
    ArchetypeIndex(archetypes.inner.len())
}

fn append_archetype(archetypes: &mut Archetypes, archetype: Archetype) {
    archetypes.inner.push(archetype);
}

fn get_archetype_for_layout_mut<'a>(archetypes: &'a mut Archetypes, layout: &EntityLayout) -> Option<&'a mut Archetype> {
    if let Some(index) = search_archetypes_for(&archetypes.search_index, layout.clone(), 0).next() {
        return Some(&mut archetypes.inner[index.0]);
    }
    None
}

pub fn get_archetype_for_layout<'a>(archetypes: &'a Archetypes, layout: &EntityLayout) -> Option<&'a Archetype> {
    if let Some(index) = search_archetypes_for(&archetypes.search_index, layout.clone(), 0).next() {
        return Some(&archetypes.inner[index.0]);
    }
    None
}

pub fn get_archetype_for_entity_location(archetypes: &Archetypes, location: EntityLocation) -> &Archetype {
    get_archetype(archetypes, location.archetype_index)
}

pub fn get_archetype(archetypes: &Archetypes, index: ArchetypeIndex) -> &Archetype {
    &archetypes.inner[index.0]
}

fn get_archetype_mut(archetypes: &mut Archetypes, index: ArchetypeIndex) -> &mut Archetype {
    &mut archetypes.inner[index.0]
}

pub struct ArchetypeIterator<'a> {
    indicies: Box<dyn Iterator<Item = ArchetypeIndex> + 'a>,
    world: &'a World,
}

pub fn create_archetype_iterator<'a>(world: &'a World, indicies: impl Iterator<Item = ArchetypeIndex> + 'a) -> ArchetypeIterator {
    ArchetypeIterator {
        indicies: Box::new(indicies),
        world,
    }
}


impl<'a> Iterator for ArchetypeIterator<'a> {
    type Item = &'a Archetype;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(index) = self.indicies.next() {
            return Some(get_archetype(&self.world.archetypes, index))
        }
        None
    }
}

pub struct Archetype {
    pub layout: EntityLayout,
    pub index: ArchetypeIndex,
    pub chunks: ComponentChunks
}

fn create_archetype(layout: EntityLayout, index: ArchetypeIndex) -> Archetype {
    Archetype { layout, index, chunks: create_component_chunks() }
}

pub fn move_to_next_archetype_entity_location(archetypes: &mut Archetypes, layout: &EntityLayout)  -> ArchetypeEntityLocation {
    let archetype = get_archetype_for_layout(archetypes, &layout).unwrap();
        
    if are_chunks_full(&archetype.chunks) {
        add_new_chunk(archetypes, layout);
    } else {
        increment_current_component_index(archetypes, layout);        
    }

    let archetype = get_archetype_for_layout(archetypes, &layout).unwrap();
    let current_chunk_index = current_chunk_index(&archetype.chunks);
    let chunk = get_chunk(&archetype.chunks, current_chunk_index);        
    create_archetype_entity_location(chunk.chunk_index, chunk.current_component_index)
}

fn increment_current_component_index(archetypes: &mut Archetypes, layout: &EntityLayout) {
    let archetype = get_archetype_for_layout_mut(archetypes, &layout).unwrap();
    let current_chunk_index = current_chunk_index(&archetype.chunks);
    let chunk = get_chunk_mut(&mut archetype.chunks, current_chunk_index);
    chunk.current_component_index.increment();
}

fn add_new_chunk(archetypes: &mut Archetypes, layout: &EntityLayout) {
    let archetype = get_archetype_for_layout(archetypes, &layout).unwrap();
    let index = next_chunk_index(&archetype.chunks);
    let chunk = component_chunk(&mut archetypes.chunk_pool, index, deduce_chunk_layout(layout));
    let archetype = get_archetype_for_layout_mut(archetypes, &layout).unwrap();
    add_chunk(&mut archetype.chunks, chunk);
}

pub fn move_to_previous_archetype_entity_location(archetypes: &mut Archetypes, archetype: ArchetypeIndex) {
    let chunks = &mut get_archetype_mut(archetypes, archetype).chunks;
    let current_chunk_index = current_chunk_index(chunks);
    let current_chunk = get_chunk_mut(chunks, current_chunk_index);
    
    if !current_chunk.current_component_index.is_first() {
        current_chunk.current_component_index.decrement();
        return;
    }
    let chunk = remove_last_chunk(chunks).unwrap();
    recycle_chunk(&mut archetypes.chunk_pool, chunk);
}