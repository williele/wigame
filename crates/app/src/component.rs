use std::{any::TypeId, collections::HashMap};

use util::{bit_set::BitSet, blob_sparse_set::BlobSparseSet};

use crate::entity::Entity;

pub trait Component: 'static + Send + Sync {}
impl<T: 'static + Send + Sync> Component for T {}

pub type ComponentVec = BlobSparseSet<Entity>;

#[derive(Default)]
pub struct Components {
    vecs: HashMap<TypeId, ComponentVec>,
}

impl Components {
    fn get_vec<T: Component>(&self) -> Option<&ComponentVec> {
        self.vecs.get(&TypeId::of::<T>())
    }

    pub(crate) fn get_bitset<T: Component>(&self) -> Option<&BitSet> {
        self.get_vec::<T>().map(|set| set.bitset())
    }

    pub(crate) unsafe fn get_ptr<T: Component>(&self, entity: Entity) -> Option<*mut u8> {
        self.get_vec::<T>().and_then(|vec| vec.get_ptr(entity))
    }

    pub(crate) fn insert<T: Component>(&mut self, entity: Entity, component: T) {
        let type_id = TypeId::of::<T>();
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
        let type_id = TypeId::of::<T>();
        if let Some(vec) = self.vecs.get_mut(&type_id) {
            vec.remove(entity);
        }
    }

    pub(crate) fn remove_raw(&mut self, type_id: &TypeId, entity: Entity) {
        if let Some(vec) = self.vecs.get_mut(type_id) {
            vec.remove(entity);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::entity::Entities;

    use super::*;

    #[derive(Debug)]
    struct Foo(i32);

    #[test]
    fn components() {
        let mut entities = Entities::default();
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
            components.remove_raw(&TypeId::of::<Foo>(), b);
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
