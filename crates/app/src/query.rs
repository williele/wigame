use util::bit_set::BitSet;

use crate::{Component, Components, Entity};

pub struct Query<'a> {
    components: &'a Components,
    pub bitset: BitSet,
}

impl<'a> Query<'a> {
    pub fn new<C: Component>(components: &'a Components) -> Self {
        let bitset: BitSet = if let Some(bitset) = components.get_bitset::<C>() {
            bitset.clone()
        } else {
            BitSet::new()
        };
        Query { components, bitset }
    }

    pub fn with<C: Component>(&mut self) -> &mut Self {
        if let Some(bitset) = self.components.get_bitset::<C>() {
            self.bitset.intersect_with(bitset);
        }
        self
    }

    pub fn without<C: Component>(&mut self) -> &mut Self {
        if let Some(bitset) = self.components.get_bitset::<C>() {
            self.bitset.symmetric_difference_with(bitset);
        }
        self
    }

    pub fn or_with<C: Component>(&mut self) -> &mut Self {
        if let Some(bitset) = self.components.get_bitset::<C>() {
            self.bitset.union_with(bitset);
        }
        self
    }

    pub fn vec(&self) -> Vec<Entity> {
        self.bitset
            .into_iter()
            .map(|e| Entity::new(e as u32, 0))
            .collect()
    }
}
