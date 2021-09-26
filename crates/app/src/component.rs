use std::{
    any::{type_name, TypeId},
    collections::HashMap,
    fmt::Display,
    hash::Hash,
};

use util::{bit_set::BitSet, blob_sparse_set::BlobSparseSet};

use crate::entity::Entity;

pub trait Component: 'static + Send + Sync {}
impl<T: 'static + Send + Sync> Component for T {}

#[derive(Debug, Clone, Copy, Eq, PartialOrd, Ord)]
pub struct ComponentTypeId {
    type_id: TypeId,
    #[cfg(debug_assertions)]
    name: &'static str,
}

impl Hash for ComponentTypeId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.type_id.hash(state)
    }
}

impl PartialEq for ComponentTypeId {
    fn eq(&self, other: &Self) -> bool {
        self.type_id.eq(&other.type_id)
    }
}

impl Display for ComponentTypeId {
    #[cfg(debug_assertions)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }

    #[cfg(not(debug_assertions))]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.type_id)
    }
}

impl ComponentTypeId {
    pub fn of<T>() -> Self
    where
        T: Component,
    {
        Self {
            type_id: TypeId::of::<T>(),
            #[cfg(debug_assertions)]
            name: type_name::<T>(),
        }
    }
}

pub type ComponentVec = BlobSparseSet<Entity>;

#[derive(Default)]
pub struct Components {
    vecs: HashMap<ComponentTypeId, ComponentVec>,
}

impl Components {
    fn get_vec<T: Component>(&self) -> Option<&ComponentVec> {
        self.vecs.get(&ComponentTypeId::of::<T>())
    }

    pub(crate) fn get_bitset<T: Component>(&self) -> Option<&BitSet> {
        self.get_vec::<T>().map(|set| set.bitset())
    }

    pub(crate) unsafe fn get_ptr<T: Component>(&self, entity: Entity) -> Option<*mut u8> {
        self.get_vec::<T>().and_then(|vec| vec.get_ptr(entity))
    }

    pub(crate) fn insert<T: Component>(&mut self, entity: Entity, component: T) {
        let type_id = ComponentTypeId::of::<T>();
        let vec = if let Some(vec) = self.vecs.get_mut(&type_id) {
            vec
        } else {
            let vec = ComponentVec::of::<T>(0);
            self.vecs.insert(type_id.clone(), vec);
            self.vecs.get_mut(&type_id).unwrap()
        };
        unsafe { vec.insert_type::<T>(entity, component) }
    }

    pub(crate) fn remove<T: Component>(&mut self, entity: Entity) {
        let type_id = ComponentTypeId::of::<T>();
        if let Some(vec) = self.vecs.get_mut(&type_id) {
            vec.remove(entity);
        }
    }

    pub(crate) fn remove_raw(&mut self, type_id: &ComponentTypeId, entity: Entity) {
        if let Some(vec) = self.vecs.get_mut(type_id) {
            vec.remove(entity);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::entity::EntityAllocator;

    use super::*;

    #[derive(Debug)]
    struct Foo(i32);

    #[test]
    fn components() {
        let mut entities = EntityAllocator::default();
        let mut components = Components::default();

        let a = entities.alloc();
        let b = entities.alloc();
        let c = entities.alloc();
        components.insert(a, Foo(0));
        components.insert(b, Foo(1));
        components.insert(c, Foo(2));

        unsafe {
            println!(
                "{:?}",
                components
                    .get_ptr::<Foo>(a)
                    .and_then(|ptr| ptr.cast::<Foo>().as_ref())
            );
            println!(
                "{:?}",
                components
                    .get_ptr::<Foo>(b)
                    .and_then(|ptr| ptr.cast::<Foo>().as_ref())
            );
            println!(
                "{:?}",
                components
                    .get_ptr::<Foo>(c)
                    .and_then(|ptr| ptr.cast::<Foo>().as_ref())
            );
            println!("{:?}", components.get_bitset::<Foo>());

            println!("----------------");
            components.remove_raw(&ComponentTypeId::of::<Foo>(), b);
            println!(
                "{:?}",
                components
                    .get_ptr::<Foo>(a)
                    .and_then(|ptr| ptr.cast::<Foo>().as_ref())
            );
            println!(
                "{:?}",
                components
                    .get_ptr::<Foo>(b)
                    .and_then(|ptr| ptr.cast::<Foo>().as_ref())
            );
            println!(
                "{:?}",
                components
                    .get_ptr::<Foo>(c)
                    .and_then(|ptr| ptr.cast::<Foo>().as_ref())
            );
            println!("{:?}", components.get_bitset::<Foo>());
        }
    }
}
