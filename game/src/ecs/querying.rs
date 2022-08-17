use std::marker::PhantomData;
use super::*;
pub trait QueryIterable<T> : Iterator {
}

pub struct QueryIterator<'a, T: View<'a>> {
    data: PhantomData<T>,
    archetypes: ArchetypeIterator<'a>,
    chunks: Option<ArchetypeComponentIterator<'a>>
}

fn create_query_iterator<'a, T: View<'a>>(world: &'a World, archetype_indicies: impl Iterator<Item = ArchetypeIndex> + 'a) -> QueryIterator<'a, T> {
    QueryIterator::<'a, T> {
        data: PhantomData::default(),
        archetypes: create_archetype_iterator(world, archetype_indicies),
        chunks: None
    }
}

impl<'a, T: View<'a, Fetch = T>> QueryIterable<T> for QueryIterator<'a, T> {
}

impl<'a, T: View<'a, Fetch = T>> Iterator for QueryIterator<'a, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(chunk_iter) = self.chunks.as_mut() {
            if let Some((chunk, component_index)) = chunk_iter.next() {
                return Some(T::fetch(chunk, component_index));
            } else {
                self.chunks = None;
            }
        }
        if let Some(archetype) = self.archetypes.next() {
            self.chunks = Some(archetype.chunks.component_iter());
            return self.next();
        }
        None
    }
}

pub trait View<'a> {
    type Fetch;
    fn fetch(chunk: &'a ComponentChunk, component_index: ComponentIndex) -> Self::Fetch;
    
}

pub fn iterate_query<'a, A:Component, B:Component>(world: &'a World) ->  QueryIterator<'a, (&'a A, &'a B)> {
    let layout = create_entity_layout_from_component::<A>();
    let layout = clone_entity_layout_and_add_component::<B>(&layout);
    let filter = any_component_filter(layout.components);
    let archetypes = search_archetypes_for(&world.archetypes.search_index, filter, 0);
    create_query_iterator(world, archetypes)
}

impl<'a, A:Component, B:Component> View<'a> for (&'a A, &'a B) {
    type Fetch = (&'a A, &'a B);
    fn fetch(chunk: &'a ComponentChunk, component_index: ComponentIndex) -> Self::Fetch {
        let a = A::fetch(chunk, component_index);
        let b = B::fetch(chunk, component_index);
        (a, b)
    }
}

/*
pub fn component<T>() -> bool {
    todo!()
}
*/


pub trait LayoutFilter {
    fn matches_layout(&self, components: &[ComponentTypeId]) -> FilterResult;
}

pub enum FilterResult {
    Match(bool)
}

impl FilterResult {
    pub fn is_pass(&self) -> bool {
        match self {
            FilterResult::Match(pass) => *pass,
        }
    }   
}

impl LayoutFilter for EntityLayout {
    fn matches_layout(&self, components: &[ComponentTypeId]) -> FilterResult {
        FilterResult::Match(
            components.len() == self.components.len()
                && self.components.iter().all(|t| components.contains(t)),
        )
    }
}

struct AnyComponentFilter(Vec<ComponentTypeId>);

fn any_component_filter(components: Vec<ComponentTypeId>) -> AnyComponentFilter {
    AnyComponentFilter(components)
}
impl LayoutFilter for AnyComponentFilter {
    fn matches_layout(&self, components: &[ComponentTypeId]) -> FilterResult {
        FilterResult::Match(self.0.iter().all(|t| components.contains(t)))
    }
}
