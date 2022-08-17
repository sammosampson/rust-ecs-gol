use std::{
    alloc::*,
    any::*,
    iter::*,
    slice::*,
    mem::*,
    marker::*,
    collections::*,
};

use gol_engine::gol_assert;

use super::*;

#[derive(Copy, Clone)]
pub struct ChunkIndex(usize);

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct ComponentIndex(usize);

impl ComponentIndex {
    pub fn increment(&mut self) {
        self.0 += 1;
    }
    
    pub fn decrement(&mut self) {
        self.0 -= 1;
    }

    pub fn is_first(&self) -> bool {
        self.0 == 0
    }
}

const CHUNK_SIZE: usize = 16384;
const CHUNK_ALIGN: usize = 1;

struct ComponentChunkLayoutStream {
    component_type_id: ComponentTypeId,
    offset: usize,
    stride: usize
}

fn component_chunk_layout_stream(component_type_id: ComponentTypeId, offset: usize, stride: usize) -> ComponentChunkLayoutStream {
    ComponentChunkLayoutStream { 
        component_type_id,
        offset,
        stride
    }
}

fn deduce_component_chunk_layout_stream(
    offset: &mut usize,
    max_slots: usize,
    component_type_id: ComponentTypeId,
    stride: usize
 ) -> ComponentChunkLayoutStream {
    let width = stride * max_slots;
    let stream = component_chunk_layout_stream(component_type_id, *offset, stride);
    *offset += width;
    stream
}

fn get_offset_in_chunk_stream(stream: &ComponentChunkLayoutStream, index: ComponentIndex) -> usize {
    stream.offset + (index.0 * stream.stride)
}

pub struct ComponentChunkLayout {
    max_slots: usize,
    streams: Vec<ComponentChunkLayoutStream>
}

fn component_chunk_layout(max_slots: usize, streams: Vec<ComponentChunkLayoutStream>) -> ComponentChunkLayout {
    ComponentChunkLayout { max_slots, streams }
}

pub fn deduce_chunk_layout(layout: &EntityLayout) -> ComponentChunkLayout {
    let max_slots = CHUNK_SIZE / get_layout_size(layout);
    component_chunk_layout(max_slots, deduce_component_chunk_layout_streams(layout, max_slots))
}

fn deduce_component_chunk_layout_streams(layout: &EntityLayout, max_slots: usize) -> Vec<ComponentChunkLayoutStream> {
    let mut offset = 0;
    let mut streams = Vec::<ComponentChunkLayoutStream>::default();

    for (component_type_id, component_size) in component_type_and_size_iter(layout) {
        let component_stream = deduce_component_chunk_layout_stream(
            &mut offset,
            max_slots,
            *component_type_id,
            *component_size);
        streams.push(component_stream);
    }

    streams
}

fn get_chunk_stream_for_component(component_type_id: ComponentTypeId, chunk_layout: &ComponentChunkLayout) -> &ComponentChunkLayoutStream {
    chunk_layout.streams
        .iter()
        .filter(|stream| stream.component_type_id == component_type_id)
        .last()
        .unwrap()
}

fn get_chunk_stream_offset_for_component(component_type: ComponentTypeId, chunk_layout: &ComponentChunkLayout, index: ComponentIndex) -> usize {
    let stream = get_chunk_stream_for_component(component_type, chunk_layout);
    get_offset_in_chunk_stream(stream, index)
}

pub struct ComponentChunk {
    storage: *mut u8,
    pub chunk_index: ChunkIndex,
    pub current_component_index: ComponentIndex,
    chunk_layout: ComponentChunkLayout
}

pub fn component_chunk(chunk_pool: &mut ComponentChunkPool, chunk_index: ChunkIndex, chunk_layout: ComponentChunkLayout) -> ComponentChunk {
    if let Some(mut chunk) = chunk_pool.recycled.pop() {
        chunk.chunk_index = chunk_index;
        chunk.chunk_layout = chunk_layout;
        chunk.current_component_index = ComponentIndex(0);
        return chunk
    }

    return ComponentChunk { 
        storage: allocate_chunk_storage(),
        chunk_index,
        current_component_index: ComponentIndex(0),
        chunk_layout
    }
}

fn is_chunk_full(chunk: &ComponentChunk) -> bool {
    chunk.current_component_index.0 == chunk.chunk_layout.max_slots - 1
}

pub fn store_component_at_location<C>(chunks: &ComponentChunks, location: ArchetypeEntityLocation, component: C)
where C: Component {
    let component_type_id = component_type_of::<C>(); 
    let chunk = get_chunk(chunks, location.chunk);
    let offset = get_chunk_stream_offset_for_component(component_type_id, &chunk.chunk_layout, chunk.current_component_index);
    store_component_in_chunk_at_offset(chunk, offset, component);
}

fn store_component_in_chunk_at_offset<C>(chunk: &ComponentChunk, offset: usize, component: C) where C:Component {
    unsafe {
        let storage_location = chunk.storage.add(offset) as *mut C;
        *storage_location = component;
    }
}

fn allocate_chunk_storage() -> *mut u8 {
    let layout = Layout::from_size_align(CHUNK_SIZE, CHUNK_ALIGN);
    let storage = unsafe { alloc(layout.unwrap()) };
    storage
}

#[derive(Default)]
pub struct ComponentChunkPool {
    recycled: Vec<ComponentChunk>
}

#[derive(Default)]
pub struct ComponentChunks {
    inner: Vec<ComponentChunk>
}

pub struct ArchetypeComponentIterator<'a> {
    chunks: Iter<'a, ComponentChunk>,
    current: Option<(&'a ComponentChunk, ComponentIndex)>
}

impl<'a> Iterator for ArchetypeComponentIterator<'a> {
    type Item = (&'a ComponentChunk, ComponentIndex);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(mut value) = self.current {
            if value.1 == value.0.current_component_index { 
                self.current = None;
            } else {
                value.1.increment();
                self.current = Some((value.0, value.1));
            }
            return Some(value);
        }

        if let Some(chunk) = self.chunks.next() {
            self.current = Some((chunk, ComponentIndex(0)));
            return self.next();
        }
        None
    }
}

impl ComponentChunks {
    pub fn component_iter(&self) -> ArchetypeComponentIterator {
        ArchetypeComponentIterator { chunks: self.inner.iter().clone(), current: None }
    }
}

pub fn create_component_chunks() -> ComponentChunks {
    ComponentChunks::default()
}

pub fn recycle_chunk(chunk_pool: &mut ComponentChunkPool, chunk: ComponentChunk) {
    chunk_pool.recycled.push(chunk);
}

pub fn add_chunk(chunks: &mut ComponentChunks, chunk: ComponentChunk) {
    chunks.inner.push(chunk)
}

pub fn remove_last_chunk(chunks: &mut ComponentChunks) -> Option<ComponentChunk> {
    chunks.inner.pop()
}

pub fn get_chunk_mut(chunks: &mut ComponentChunks, index: ChunkIndex) -> &mut ComponentChunk {
    &mut chunks.inner[index.0]
}

pub fn get_chunk(chunks: &ComponentChunks, index: ChunkIndex) -> &ComponentChunk {
    &chunks.inner[index.0]
}

pub fn are_chunks_full(chunks: &ComponentChunks) -> bool {
    if let Some(chunk) = chunks.inner.last() {
        return is_chunk_full(chunk)
    }
    true
}

pub fn current_chunk_index(chunks: &ComponentChunks) -> ChunkIndex {
    ChunkIndex(chunks.inner.len() - 1) 
}

pub fn next_chunk_index(chunks: &ComponentChunks) -> ChunkIndex {
    ChunkIndex(chunks.inner.len()) 
}

#[derive(Clone, PartialEq, Eq)]
pub struct EntityLayout { 
    pub components: Vec<ComponentTypeId>,
    component_sizes: Vec<usize>
}

pub fn create_entity_layout_from_component<C>() -> EntityLayout
where C: Component {
    create_entity_layout_from_component_and_size(component_type_of::<C>(), size_of::<C>())
}

fn create_entity_layout_from_component_and_size(component: ComponentTypeId, size: usize) -> EntityLayout {
    let mut components = Vec::<ComponentTypeId>::default();
    components.push(component);
    let mut component_sizes = Vec::<usize>::default();
    component_sizes.push(size);
    EntityLayout {
        components,
        component_sizes,
    }
}

fn get_layout_size(layout: &EntityLayout) -> usize {
    layout.component_sizes.iter().sum()
}

pub fn copy_layout_components(layout: &EntityLayout) -> Copied<Iter<ComponentTypeId>> {
    layout.components.iter().copied()
}

fn component_type_and_size_iter<'a>(layout: &'a EntityLayout) -> Zip<Iter<'a, ComponentTypeId>, Iter<'a, usize>> {
    layout.components.iter().zip(&layout.component_sizes)
}

pub fn clone_entity_layout_and_add_component<C>(layout: &EntityLayout) -> EntityLayout
where C: Component {  
    let mut cloned = layout.clone();
    cloned.components.push(component_type_of::<C>());
    cloned.component_sizes.push(size_of::<C>());
    cloned
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct ComponentTypeId {
    type_id: TypeId,
    name: &'static str,
}

#[derive(Default)]
pub struct ComponentStorages {    
    inner: HashMap<ComponentTypeId, Box<dyn UnknownComponentStorage>>
}

pub fn component_storages() -> ComponentStorages {
    ComponentStorages::default()
}


pub fn get_component_storage(storages: &ComponentStorages, type_id: ComponentTypeId) -> &Box<dyn UnknownComponentStorage> {
    gol_assert!(storages.inner.contains_key(&type_id));
    storages.inner.get(&type_id).unwrap()
}

pub fn set_component_storage_if_not_set_already<C: Component>(storages: &mut ComponentStorages) {
    let type_id = component_type_of::<C>(); 
    if !storages.inner.contains_key(&type_id) { 
        storages.inner.insert(type_id, Box::new(create_packed_component_storage::<C>()));
    }
}

pub fn move_components(
    storages: &mut ComponentStorages,
    source_layout: EntityLayout,
    source_archetype: &Archetype,
    source_archetype_entity_location: ArchetypeEntityLocation,
    target_archetype: &Archetype,
    target_archetype_entity_location: ArchetypeEntityLocation
) {
    for type_id in &source_layout.components {
        let component_storage = get_component_storage(storages, *type_id);
        component_storage.move_component(
            &source_archetype.chunks, 
            source_archetype_entity_location.chunk, 
            source_archetype_entity_location.component, 
            &target_archetype.chunks,
            target_archetype_entity_location.chunk
        );
    }
}

pub trait ComponentStorage<'a, T: Component>: Sized + Send + Sync {}

pub trait Component: 'static + Sized + Send + Sync {
    type Storage: for<'a> ComponentStorage<'a, Self>;
}

impl<T: 'static + Sized + Send + Sync> Component for T {
    type Storage = PackedComponentStorage<T>;
}

pub trait UnknownComponentStorage {
    fn move_component(&self, source_chunks: &ComponentChunks, source_chunk_index: ChunkIndex, source_component_index: ComponentIndex, target_chunks: &ComponentChunks, target_chunk_index: ChunkIndex);
}

pub struct PackedComponentStorage<C: Component> {
    _data: PhantomData<C>,
    component_type_id: ComponentTypeId
}

fn create_packed_component_storage<C: Component>() -> PackedComponentStorage<C> {
    PackedComponentStorage::<C> {
        _data: PhantomData::default(),
        component_type_id: component_type_of::<C>(),
    }
}

impl <'a, C> ComponentStorage<'a, C> for PackedComponentStorage<C> 
where C: Component {
}

impl <T: Component> UnknownComponentStorage for PackedComponentStorage<T> {
    fn move_component(&self, source_chunks: &ComponentChunks, source_chunk_index: ChunkIndex, source_component_index: ComponentIndex, target_chunks: &ComponentChunks, target_chunk_index: ChunkIndex) {
        let source_chunk = get_chunk(source_chunks, source_chunk_index);
        let source_head_chunk = get_chunk(source_chunks, current_chunk_index(source_chunks));
        let target_chunk = get_chunk(target_chunks, target_chunk_index);
        let component = self.swap_remove_component_in_chunk(source_chunk, source_head_chunk, source_component_index);
        self.add_component(target_chunk, target_chunk.current_component_index, component);
    }
}

impl <C> PackedComponentStorage<C> 
where C: Component {        
    fn add_component(&self, chunk: &ComponentChunk, component_index: ComponentIndex, component: C) {
        let offset = get_chunk_stream_offset_for_component(self.component_type_id, &chunk.chunk_layout, component_index);
        add_component_in_chunk_storage(chunk, offset, component);
    }

    fn swap_remove_component_in_chunk(&self, chunk: &ComponentChunk, head_chunk: &ComponentChunk, component_index: ComponentIndex) -> C {
        let item_offset = get_chunk_stream_offset_for_component(self.component_type_id, &chunk.chunk_layout, component_index);
        let head_offset = get_chunk_stream_offset_for_component(self.component_type_id, &head_chunk.chunk_layout, head_chunk.current_component_index);
        let read = swap_read_component_in_chunk_storage(chunk, chunk, item_offset, head_offset);
        read
    }      
}

pub trait ReadFetch<'a, T> {
    type Data;
    fn fetch(chunk: &'a ComponentChunk, component_index: ComponentIndex) -> Self::Data;
}      

impl<'a, C:Component> ReadFetch<'a, C> for C {
    type Data = &'a C;
    fn fetch(chunk: &'a ComponentChunk, component_index: ComponentIndex) -> Self::Data {
        let offset = get_chunk_stream_offset_for_component(component_type_of::<C>(), &chunk.chunk_layout, component_index);
        get_component_from_chunk_storage(chunk, offset)    
    }
}

fn add_component_in_chunk_storage<C:Component>(chunk: &ComponentChunk, index: usize, component: C) {
    let removed = unsafe { 
        let pointer = chunk.storage.add(index) as *mut C;
        *pointer = component
    };
    removed
}

fn swap_read_component_in_chunk_storage<C:Component>(source_chunk: &ComponentChunk, target_chunk: &ComponentChunk, source_offset: usize, target_offset: usize) -> C {
    let removed = unsafe { 
        let src_pointer = source_chunk.storage.add(source_offset) as *mut C;
        let target_pointer = target_chunk.storage.add(target_offset) as *mut C;
        if src_pointer != target_pointer {
            std::ptr::swap(src_pointer, target_pointer);
        }
        std::ptr::read(target_pointer)
    };
    removed
}

fn get_component_from_chunk_storage<C:Component>(chunk: &ComponentChunk, offset: usize) -> &C {
    unsafe {
        (chunk.storage.add(offset) as *mut C).as_ref().unwrap()
    }
}


pub fn component_type_of<T: Component>() -> ComponentTypeId {
    ComponentTypeId {
        type_id: TypeId::of::<T>(),
        name: type_name::<T>(),
    }
}