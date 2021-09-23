use std::{any::TypeId, collections::HashSet, fmt::Debug};

use util::{bit_set::BitSet, sparse_set::SparseIndex};

use crate::Component;

#[derive(Clone, Copy)]
pub struct Entity {
    id: u32,
    pub generation: u32,
}

impl Debug for Entity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}v{}", self.id, self.generation)
    }
}

impl Entity {
    pub fn new(id: u32, generation: u32) -> Self {
        Entity { id, generation }
    }

    pub fn id(&self) -> u32 {
        self.id
    }
}

impl SparseIndex for Entity {
    fn get_sparse_index(&self) -> usize {
        self.id as usize
    }
}

pub(crate) struct EntityEntry {
    is_live: bool,
    generation: u32,
    components: Option<HashSet<TypeId>>,
}

#[derive(Default)]
pub(crate) struct Entities {
    entries: Vec<EntityEntry>,
    bitset: BitSet,
    pending: Vec<u32>,
}

impl Entities {
    #[inline]
    pub(crate) fn get_entity(&self, id: u32) -> Option<Entity> {
        self.entries
            .get(id as usize)
            .and_then(|entry| entry.is_live.then(|| Entity::new(id, entry.generation)))
    }

    #[inline]
    pub(crate) fn get_bitset(&self) -> &BitSet {
        &self.bitset
    }

    pub(crate) fn add_component<T: Component>(&mut self, entity: Entity) {
        if self.is_live(entity) {
            let index = entity.id as usize;
            let components = self.entries[index]
                .components
                .get_or_insert(Default::default());
            components.insert(TypeId::of::<T>());
        }
    }

    pub(crate) fn remove_commponent<T: Component>(&mut self, entity: Entity) {
        if self.is_live(entity) {
            let index = entity.id as usize;
            if let Some(components) = &mut self.entries[index].components {
                components.remove(&TypeId::of::<T>());
            }
        }
    }

    pub fn alloc(&mut self) -> Entity {
        match self.pending.pop() {
            Some(id) => {
                let index = id as usize;
                let entry = &mut self.entries[index];
                entry.generation += 1;
                entry.is_live = true;
                self.bitset.insert(index);
                Entity::new(id, entry.generation)
            }
            None => {
                let id = self.entries.len();
                self.entries.push(EntityEntry {
                    is_live: true,
                    components: None,
                    generation: 0,
                });
                self.bitset.insert(id);
                Entity::new(id as u32, 0)
            }
        }
    }

    pub fn delloc(&mut self, entity: Entity) -> Option<HashSet<TypeId>> {
        if self.is_live(entity) {
            let index = entity.id as usize;
            self.entries[index].is_live = false;
            self.pending.push(entity.id);
            self.bitset.remove(index);
            self.entries[index].components.take()
        } else {
            None
        }
    }

    pub fn is_live(&self, entity: Entity) -> bool {
        let index = entity.id as usize;
        index < self.entries.len()
            && self.entries[index].generation == entity.generation
            && self.entries[index].is_live
    }
}
