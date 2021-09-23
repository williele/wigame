use std::{any::TypeId, collections::HashMap};

use util::{bit_set::BitSet, blob_sparse_set::BlobSparseSet, parking_lot::RwLock};

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

    pub fn get<T: Component>(&self, entity: Entity) -> Option<&RwLock<T>> {
        self.get_vec::<T>()
            .and_then(|vec| vec.get::<RwLock<T>>(entity))
    }

    pub fn get_bitset<T: Component>(&self) -> Option<&BitSet> {
        self.get_vec::<T>().map(|set| set.bitset())
    }

    pub(crate) fn insert<T: Component>(&mut self, entity: Entity, component: T) {
        let type_id = TypeId::of::<T>();
        let vec = if let Some(vec) = self.vecs.get_mut(&type_id) {
            vec
        } else {
            let vec = ComponentVec::of::<RwLock<T>>(0);
            self.vecs.insert(type_id.clone(), vec);
            self.vecs.get_mut(&type_id).unwrap()
        };

        unsafe { vec.insert_type::<RwLock<T>>(entity, RwLock::new(component)) }
    }

    pub(crate) fn remove(&mut self, type_id: &TypeId, entity: Entity) {
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

        println!("{:?}", components.get::<Foo>(a));
        println!("{:?}", components.get::<Foo>(b));
        println!("{:?}", components.get::<Foo>(c));
        println!("{:?}", components.get_bitset::<Foo>());

        println!("----------------");
        components.remove(&TypeId::of::<Foo>(), b);
        println!("{:?}", components.get::<Foo>(a));
        println!("{:?}", components.get::<Foo>(b));
        println!("{:?}", components.get::<Foo>(c));
        println!("{:?}", components.get_bitset::<Foo>());
    }
}
