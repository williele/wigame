use util::{anymap::AnyMap, bit_set::BitSet, parking_lot::RwLock, sparse_set::SparseSet};

use crate::entity::Entity;

pub trait Component: 'static + Send + Sync {}
impl<T: 'static + Send + Sync> Component for T {}

pub type ComponentSet<T> = SparseSet<Entity, RwLock<T>>;

#[derive(Debug, Default)]
pub struct Components {
    set: AnyMap,
}

impl Components {
    pub fn get_set<T: Component>(&self) -> Option<&ComponentSet<T>> {
        self.set.get::<ComponentSet<T>>()
    }

    pub fn get<T: Component>(&self, entity: Entity) -> Option<&RwLock<T>> {
        self.get_set::<T>().and_then(|set| set.get(entity))
    }

    pub fn get_unchecked<T: Component>(&self, entity: Entity) -> &RwLock<T> {
        self.get_set::<T>().unwrap().get_unchecked(entity)
    }

    pub fn get_bitset<T: Component>(&self) -> Option<&BitSet> {
        self.get_set::<T>().map(|set| set.bitset())
    }

    pub(crate) fn insert<T: Component>(&mut self, entity: Entity, component: T) {
        let set = if let Some(set) = self.set.get_mut::<ComponentSet<T>>() {
            set
        } else {
            let set = ComponentSet::<T>::default();
            self.set.insert(set);
            self.set.get_mut::<ComponentSet<T>>().unwrap()
        };

        set.insert(entity, RwLock::new(component));
    }
}
