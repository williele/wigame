use std::fmt::Debug;

use util::{bit_set::BitSet, sparse_set::SparseIndex};

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
}

impl SparseIndex for Entity {
    fn get_sparse_index(&self) -> usize {
        self.id as usize
    }
}

pub(crate) struct EntityEntry {
    is_live: bool,
    generation: u32,
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
                    generation: 0,
                });
                self.bitset.insert(id);
                Entity::new(id as u32, 0)
            }
        }
    }

    // pub fn delloc(&mut self, entity: Entity) -> bool {
    //     if self.is_live(entity) {
    //         let index = entity.id as usize;
    //         self.entries[entity.id as usize].is_live = false;
    //         self.pending.push(entity.id);
    //         self.bitset.remove(index);
    //         true
    //     } else {
    //         false
    //     }
    // }

    // pub fn is_live(&self, entity: Entity) -> bool {
    //     let index = entity.id as usize;
    //     index < self.entries.len()
    //         && self.entries[index].generation == entity.generation
    //         && self.entries[index].is_live
    // }
}
