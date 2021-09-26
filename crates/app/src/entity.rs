use std::{
    any::TypeId,
    collections::HashSet,
    convert::TryFrom,
    fmt::Debug,
    sync::atomic::{AtomicI64, Ordering},
};

use util::{bit_set::BitSet, sparse_set::SparseIndex};

use crate::Component;

#[derive(Clone, Copy)]
pub struct Entity {
    id: u32,
    generation: u32,
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

#[derive(Debug, Clone)]
pub(crate) struct EntityEntry {
    is_live: bool,
    generation: u32,
    components: Option<HashSet<TypeId>>,
}

#[derive(Default)]
pub struct EntityAllocator {
    entries: Vec<EntityEntry>,
    bitset: BitSet,
    pending: Vec<u32>,
    cursor: AtomicI64,
    len: u32,
}

impl EntityAllocator {
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

    #[inline]
    pub fn len(&self) -> u32 {
        self.len
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

    pub(crate) fn flush(&mut self) {
        let cursor = self.cursor.get_mut();
        let current_cursor = *cursor;

        let new_cursor = if current_cursor >= 0 {
            current_cursor as usize
        } else {
            let old_len = self.entries.len();
            let new_len = old_len + -current_cursor as usize;
            self.len += -current_cursor as u32;
            self.entries.resize(
                new_len,
                EntityEntry {
                    is_live: true,
                    generation: 0,
                    components: None,
                },
            );
            for bit in old_len..new_len {
                self.bitset.insert(bit);
            }
            *cursor = 0;
            0
        };

        self.len += (self.pending.len() - new_cursor) as u32;
        for id in self.pending.drain(new_cursor..) {
            let entry = &mut self.entries[id as usize];
            entry.is_live = true;
            self.bitset.insert(id as usize);
        }
    }

    pub(crate) fn reserve(&self) -> Entity {
        let n = self.cursor.fetch_sub(1, Ordering::Relaxed);
        if n > 0 {
            let id = self.pending[(n - 1) as usize];
            Entity {
                generation: self.entries[id as usize].generation,
                id,
            }
        } else {
            Entity {
                generation: 0,
                id: u32::try_from(self.entries.len() as i64 - n).expect("too many entities"),
            }
        }
    }

    pub(crate) fn alloc(&mut self) -> Entity {
        self.len += 1;
        if let Some(id) = self.pending.pop() {
            *self.cursor.get_mut() = self.pending.len() as i64;
            self.entries[id as usize].is_live = true;
            self.bitset.insert(id as usize);
            Entity {
                generation: self.entries[id as usize].generation,
                id,
            }
        } else {
            let id = u32::try_from(self.entries.len()).expect("too many entities");
            self.entries.push(EntityEntry {
                is_live: true,
                components: None,
                generation: 0,
            });
            self.bitset.insert(id as usize);
            Entity { generation: 0, id }
        }
    }

    pub(crate) fn delloc(&mut self, entity: Entity) -> Option<HashSet<TypeId>> {
        if self.is_live(entity) {
            let entry = &mut self.entries[entity.id as usize];
            entry.is_live = false;
            entry.generation += 1;

            self.pending.push(entity.id);
            self.bitset.remove(entity.id as usize);

            *self.cursor.get_mut() = self.pending.len() as i64;
            self.len -= 1;

            self.entries[entity.id as usize].components.take()
        } else {
            None
        }
    }

    pub(crate) fn is_live(&self, entity: Entity) -> bool {
        let index = entity.id as usize;
        index < self.entries.len()
            && self.entries[index].generation == entity.generation
            && self.entries[index].is_live
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entity_allocator() {
        let mut a = EntityAllocator::default();
        let e1 = a.reserve();
        let e2 = a.reserve();
        let e3 = a.reserve();

        assert_eq!(e1.id, 0);
        assert_eq!(e2.id, 1);
        assert_eq!(e3.id, 2);

        assert_eq!(a.len(), 0);
        assert_eq!(a.get_bitset().len(), 0);
        a.flush();
        assert_eq!(a.len(), 3);
        assert_eq!(a.get_bitset().len(), 3);
        assert!(a.get_bitset().contains(0));
        assert!(a.get_bitset().contains(1));
        assert!(a.get_bitset().contains(2));

        let e4 = a.alloc();
        assert_eq!(e4.id, 3);
        assert_eq!(a.len(), 4);
        assert_eq!(a.get_bitset().len(), 4);
        assert!(a.get_bitset().contains(3));

        a.delloc(e2);
        assert_eq!(a.len(), 3);
        assert_eq!(a.get_bitset().len(), 3);
        assert!(!a.get_bitset().contains(1));

        let e5 = a.reserve();
        assert_eq!(e5.id, 1);
        a.flush();
        assert_eq!(a.len(), 4);
        assert_eq!(a.get_bitset().len(), 4);
        assert!(a.get_bitset().contains(1));
    }
}
