use std::fmt::Debug;

use util::sparse_set::SparseIndex;

#[derive(Clone, Copy)]
pub struct Entity {
    id: u32,
    pub generation: u32,
}

impl Debug for Entity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ent{}", self.id)
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

struct EntityEntry {
    is_live: bool,
    generation: u32,
}

#[derive(Default)]
pub(crate) struct Entities {
    entries: Vec<EntityEntry>,
    pending: Vec<u32>,
}

impl Entities {
    pub fn alloc(&mut self) -> Entity {
        match self.pending.pop() {
            Some(id) => {
                let index = id as usize;
                self.entries[index].generation += 1;
                self.entries[index].is_live = true;
                Entity::new(id, self.entries[index].generation)
            }
            None => {
                self.entries.push(EntityEntry {
                    is_live: true,
                    generation: 0,
                });
                Entity::new(self.entries.len() as u32 - 1, 0)
            }
        }
    }

    // pub fn delloc(&mut self, entity: Entity) -> bool {
    //     if self.is_live(entity) {
    //         self.entries[entity.id as usize].is_live = true;
    //         self.pending.push(entity.id);
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
