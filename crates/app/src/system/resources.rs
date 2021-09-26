// https://github.com/amethyst/legion/blob/master/src/internals/systems/resources.rs
use std::{
    any::{type_name, TypeId},
    collections::{hash_map::Entry, HashMap},
    fmt::Display,
    hash::Hash,
    marker::PhantomData,
};

use util::{
    atomic_refcell::{AtomicRef, AtomicRefCell, AtomicRefMut},
    downcast_rs::{impl_downcast, Downcast},
};

use crate::{Read, Write};

#[derive(Debug, Clone, Copy, Eq, PartialOrd, Ord)]
pub struct ResourceTypeId {
    type_id: TypeId,
    #[cfg(debug_assertions)]
    name: &'static str,
}

impl Hash for ResourceTypeId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.type_id.hash(state)
    }
}

impl PartialEq for ResourceTypeId {
    fn eq(&self, other: &Self) -> bool {
        self.type_id.eq(&other.type_id)
    }
}

impl Display for ResourceTypeId {
    #[cfg(debug_assertions)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }

    #[cfg(not(debug_assertions))]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.type_id)
    }
}

impl ResourceTypeId {
    pub fn of<T>() -> Self
    where
        T: Resource,
    {
        Self {
            type_id: TypeId::of::<T>(),
            #[cfg(debug_assertions)]
            name: type_name::<T>(),
        }
    }
}

pub trait Resource: 'static + Downcast {}
impl<T> Resource for T where T: 'static {}
impl_downcast!(Resource);

pub trait ResourceSet<'a> {
    type Item: 'a;

    unsafe fn fetch_unchecked(resources: &'a UnsafeResources) -> Self::Item;
    fn fetch_mut(resources: &'a mut Resources) -> Self::Item {
        unsafe { Self::fetch_unchecked(&resources.internal) }
    }

    fn fetch(resources: &'a Resources) -> Self::Item {
        unsafe { Self::fetch_unchecked(&resources.internal) }
    }
}

impl<'a> ResourceSet<'a> for () {
    type Item = ();

    unsafe fn fetch_unchecked(_resources: &'a UnsafeResources) -> Self::Item {}
}

impl<'a, T: Resource> ResourceSet<'a> for Read<T> {
    type Item = AtomicRef<'a, T>;

    unsafe fn fetch_unchecked(resources: &'a UnsafeResources) -> Self::Item {
        let type_id = &ResourceTypeId::of::<T>();
        resources
            .get(type_id)
            .map(|x| x.get::<T>())
            .unwrap_or_else(|| panic_nonexistent_resource(type_id))
    }
}

impl<'a, T: Resource> ResourceSet<'a> for Write<T> {
    type Item = AtomicRefMut<'a, T>;

    unsafe fn fetch_unchecked(resources: &'a UnsafeResources) -> Self::Item {
        let type_id = &ResourceTypeId::of::<T>();
        resources
            .get(&type_id)
            .map(|x| x.get_mut::<T>())
            .unwrap_or_else(|| panic_nonexistent_resource(type_id))
    }
}

fn panic_nonexistent_resource(type_id: &ResourceTypeId) -> ! {
    #[cfg(debug_assertions)]
    panic!("resource {} does not exist", type_id.name);
    #[cfg(not(debug_assertions))]
    panic!("some resource does not exist");
}

macro_rules! resource_tuple {
    ($head_ty:ident) => {
        impl_resource_tuple!($head_ty);
    };
    ($head_ty:ident, $( $tail_ty:ident ),*) => (
        impl_resource_tuple!($head_ty, $( $tail_ty ),*);
        resource_tuple!($( $tail_ty ),*);
    );
}

macro_rules! impl_resource_tuple {
    ($($ty: ident),*) => {
        #[allow(unused_parens, non_snake_case)]
        impl<'a, $($ty: ResourceSet<'a>),*> ResourceSet<'a> for ($($ty,)*)
        {
            type Item = ($($ty::Item,)*);

            unsafe fn fetch_unchecked(resources: &'a UnsafeResources) -> Self::Item {
                ($( $ty::fetch_unchecked(resources), )*)
            }
        }
    };
}
resource_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z);

pub struct ResourceCell {
    data: AtomicRefCell<Box<dyn Resource>>,
}

impl ResourceCell {
    fn new(resource: Box<dyn Resource>) -> Self {
        ResourceCell {
            data: AtomicRefCell::new(resource),
        }
    }

    fn into_inner(self) -> Box<dyn Resource> {
        self.data.into_inner()
    }

    pub fn get<T: Resource>(&self) -> AtomicRef<T> {
        let borrow = self.data.borrow();
        AtomicRef::map(borrow, |inner| inner.downcast_ref::<T>().unwrap())
    }

    pub fn get_mut<T: Resource>(&self) -> AtomicRefMut<T> {
        let borrow = self.data.borrow_mut();
        AtomicRefMut::map(borrow, |inner| inner.downcast_mut::<T>().unwrap())
    }
}

#[derive(Default)]
pub struct UnsafeResources {
    map: HashMap<ResourceTypeId, ResourceCell>,
}
unsafe impl Send for UnsafeResources {}
unsafe impl Sync for UnsafeResources {}

impl UnsafeResources {
    fn contains(&self, type_id: &ResourceTypeId) -> bool {
        self.map.contains_key(type_id)
    }

    unsafe fn entry(&mut self, type_id: ResourceTypeId) -> Entry<ResourceTypeId, ResourceCell> {
        self.map.entry(type_id)
    }

    unsafe fn insert<T: Resource>(&mut self, resource: T) {
        self.map.insert(
            ResourceTypeId::of::<T>(),
            ResourceCell::new(Box::new(resource)),
        );
    }

    unsafe fn remove(&mut self, type_id: &ResourceTypeId) -> Option<Box<dyn Resource>> {
        self.map.remove(type_id).map(|cell| cell.into_inner())
    }

    fn get(&self, type_id: &ResourceTypeId) -> Option<&ResourceCell> {
        self.map.get(type_id)
    }

    unsafe fn merge(&mut self, mut other: Self) {
        for resource in other.map.drain() {
            self.map.entry(resource.0).or_insert(resource.1);
        }
    }
}

#[derive(Default)]
pub struct Resources {
    internal: UnsafeResources,
    _not_send_sync: PhantomData<*const u8>,
}

impl Resources {
    pub(crate) fn internal(&self) -> &UnsafeResources {
        &self.internal
    }

    pub fn sync(&mut self) -> SyncResources {
        SyncResources {
            internal: &self.internal,
        }
    }

    pub fn contains<T: Resource>(&self) -> bool {
        self.internal.contains(&ResourceTypeId::of::<T>())
    }

    pub fn insert<T: Resource>(&mut self, value: T) {
        unsafe {
            self.internal.insert(value);
        }
    }

    pub fn remove<T: Resource>(&mut self) -> Option<T> {
        unsafe {
            let resource = self
                .internal
                .remove(&ResourceTypeId::of::<T>())?
                .downcast::<T>()
                .ok()?;
            Some(*resource)
        }
    }

    pub fn get<T: Resource>(&self) -> Option<AtomicRef<T>> {
        let type_id = &ResourceTypeId::of::<T>();
        self.internal.get(&type_id).map(|x| x.get::<T>())
    }

    pub fn get_mut<T: Resource>(&self) -> Option<AtomicRefMut<T>> {
        let type_id = &ResourceTypeId::of::<T>();
        self.internal.get(&type_id).map(|x| x.get_mut::<T>())
    }

    pub fn get_or_insert_with<T: Resource, F: FnOnce() -> T>(&mut self, f: F) -> AtomicRef<T> {
        let type_id = ResourceTypeId::of::<T>();
        unsafe {
            self.internal
                .entry(type_id)
                .or_insert_with(|| ResourceCell::new(Box::new((f)())))
                .get()
        }
    }

    pub fn get_mut_or_insert_with<T: Resource, F: FnOnce() -> T>(
        &mut self,
        f: F,
    ) -> AtomicRefMut<T> {
        let type_id = ResourceTypeId::of::<T>();
        unsafe {
            self.internal
                .entry(type_id)
                .or_insert_with(|| ResourceCell::new(Box::new((f)())))
                .get_mut()
        }
    }

    pub fn get_or_insert<T: Resource>(&mut self, value: T) -> AtomicRef<T> {
        self.get_or_insert_with(|| value)
    }

    pub fn get_mut_or_insert<T: Resource>(&mut self, value: T) -> AtomicRefMut<T> {
        self.get_mut_or_insert_with(|| value)
    }

    pub fn get_or_default<T: Resource + Default>(&mut self) -> AtomicRef<T> {
        self.get_or_insert_with(T::default)
    }

    pub fn get_mut_or_default<T: Resource + Default>(&mut self) -> AtomicRefMut<T> {
        self.get_mut_or_insert_with(T::default)
    }

    pub fn merge(&mut self, other: Resources) {
        unsafe {
            self.internal.merge(other.internal);
        }
    }
}

pub struct SyncResources<'a> {
    internal: &'a UnsafeResources,
}

impl<'a> SyncResources<'a> {
    pub fn get<T: Resource + Sync>(&self) -> Option<AtomicRef<T>> {
        let type_id = &ResourceTypeId::of::<T>();
        self.internal.get(&type_id).map(|x| x.get::<T>())
    }

    pub fn get_mut<T: Resource + Send>(&self) -> Option<AtomicRefMut<T>> {
        let type_id = &ResourceTypeId::of::<T>();
        self.internal.get(&type_id).map(|x| x.get_mut::<T>())
    }
}
