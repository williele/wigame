use util::bit_set::BitSet;

use crate::{Component, Components, Entity};

pub struct Query<'a> {
    components: &'a Components,
    pub bitset: Option<BitSet>,
}

impl<'a> Query<'a> {
    pub fn new<C: Component>(components: &'a Components) -> Self {
        let bitset: Option<BitSet> = if let Some(bitset) = components.get_bitset::<C>() {
            Some(bitset.clone())
        } else {
            None
        };
        Query { components, bitset }
    }

    pub fn empty(components: &'a Components) -> Self {
        Query {
            components,
            bitset: None,
        }
    }

    pub fn with<C: Component>(&mut self) -> &mut Self {
        if let Some(b) = self.components.get_bitset::<C>() {
            if let Some(a) = self.bitset.as_mut() {
                a.intersect_with(b)
            } else {
                self.bitset = Some(b.clone());
            }
        }
        self
    }

    pub fn without<C: Component>(&mut self) -> &mut Self {
        if let Some(b) = self.components.get_bitset::<C>() {
            if let Some(a) = self.bitset.as_mut() {
                a.symmetric_difference_with(b);
            }
        }
        self
    }

    pub fn or_with<C: Component>(&mut self) -> &mut Self {
        if let Some(b) = self.components.get_bitset::<C>() {
            if let Some(a) = self.bitset.as_mut() {
                a.union_with(b);
            } else {
                self.bitset = Some(b.clone());
            }
        }
        self
    }

    pub fn vec(&self) -> Vec<Entity> {
        self.bitset
            .as_ref()
            .unwrap_or(&BitSet::new())
            .into_iter()
            .map(|e| Entity::new(e as u32, 0))
            .collect()
    }
}
