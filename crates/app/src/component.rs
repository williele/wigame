use std::{any::TypeId, collections::HashMap};

use util::{bit_set::BitSet, blob_sparse_set::BlobSparseSet, parking_lot::RwLock};

use crate::entity::Entity;

// pub(crate) struct ComponentInfo {
//     type_id: TypeId,
//     layout: Layout,
//     ptr: NonNull<u8>,
//     drop: unsafe fn(*mut u8),
// }

// impl ComponentInfo {
//     unsafe fn drop_ptr<T>(ptr: *mut u8) {
//         ptr.cast::<T>().drop_in_place()
//     }

//     pub fn of<T: Component>(value: T) -> Self {
//         let mut lock = RwLock::new(value);
//         let ptr = (&mut lock as *mut RwLock<T>).cast::<u8>();
//         let layout = Layout::new::<RwLock<T>>();
//         std::mem::forget(lock);

//         ComponentInfo {
//             type_id: TypeId::of::<T>(),
//             ptr: unsafe { NonNull::new_unchecked(ptr) },
//             drop: Self::drop_ptr::<T>,
//             layout,
//         }
//     }
// }

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

    /// Safety
    /// Value and layout much be RwLock
    // pub(crate) fn insert_raw(&mut self, entity: Entity, component_info: ComponentInfo) {
    //     let vec = if let Some(vec) = self.vecs.get_mut(&component_info.type_id) {
    //         vec
    //     } else {
    //         let vec = ComponentVec::new(component_info.layout, component_info.drop, 0);
    //         self.vecs.insert(component_info.type_id.clone(), vec);
    //         self.vecs.get_mut(&component_info.type_id).unwrap()
    //     };
    //     unsafe { vec.insert(entity, component_info.ptr.as_ptr()) }
    // }

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

        println!("{:?}", components.get::<Foo>(a));
        println!("{:?}", components.get::<Foo>(b));
        println!("{:?}", components.get::<Foo>(c));
        println!("{:?}", components.get_bitset::<Foo>());

        println!("----------------");
        components.remove_raw(&TypeId::of::<Foo>(), b);
        println!("{:?}", components.get::<Foo>(a));
        println!("{:?}", components.get::<Foo>(b));
        println!("{:?}", components.get::<Foo>(c));
        println!("{:?}", components.get_bitset::<Foo>());
    }
}
