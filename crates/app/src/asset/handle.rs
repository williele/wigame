use std::{
    any::TypeId,
    cmp::Ordering,
    hash::{Hash, Hasher},
    marker::PhantomData,
};
use util::{crossbeam_channel::Sender, rand};

use crate::{Asset, RefChange};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum HandleId {
    Id(TypeId, u64),
}

impl HandleId {
    #[inline]
    pub fn random<T: Asset>() -> Self {
        HandleId::Id(TypeId::of::<T>(), rand::random())
    }

    #[inline]
    pub fn default<T: Asset>() -> Self {
        HandleId::Id(TypeId::of::<T>(), 0)
    }

    #[inline]
    pub const fn new(type_id: TypeId, id: u64) -> Self {
        HandleId::Id(type_id, id)
    }
}

impl<T: Asset> From<Handle<T>> for HandleId {
    fn from(value: Handle<T>) -> Self {
        value.id
    }
}

impl<T: Asset> From<&Handle<T>> for HandleId {
    fn from(value: &Handle<T>) -> Self {
        value.id
    }
}

#[derive(Debug)]
enum HandleType {
    Weak,
    Strong(Sender<RefChange>),
}

#[derive(Debug)]
pub struct Handle<T: Asset> {
    pub id: HandleId,
    handle_type: HandleType,
    _marker: PhantomData<fn() -> T>,
}

impl<T: Asset> Handle<T> {
    pub(crate) fn strong(id: HandleId, ref_change_sender: Sender<RefChange>) -> Self {
        ref_change_sender.send(RefChange::Increment(id)).unwrap();
        Self {
            id,
            handle_type: HandleType::Strong(ref_change_sender),
            _marker: Default::default(),
        }
    }

    pub fn weak(id: HandleId) -> Self {
        Self {
            id,
            handle_type: HandleType::Weak,
            _marker: Default::default(),
        }
    }

    pub fn is_weak(&self) -> bool {
        matches!(self.handle_type, HandleType::Weak)
    }

    pub fn is_strong(&self) -> bool {
        matches!(self.handle_type, HandleType::Strong(_))
    }

    pub fn close_weak(&self) -> Self {
        Self {
            id: self.id,
            handle_type: HandleType::Weak,
            _marker: Default::default(),
        }
    }
}

impl<T: Asset> Clone for Handle<T> {
    fn clone(&self) -> Self {
        match self.handle_type {
            HandleType::Strong(ref sender) => Handle::strong(self.id, sender.clone()),
            HandleType::Weak => Handle::weak(self.id),
        }
    }
}

impl<T: Asset> Drop for Handle<T> {
    fn drop(&mut self) {
        match self.handle_type {
            HandleType::Weak => {}
            HandleType::Strong(ref sender) => {
                let _ = sender.send(RefChange::Decrement(self.id)).unwrap();
            }
        }
    }
}

impl<T: Asset> Hash for Handle<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Hash::hash(&self.id, state);
    }
}

impl<T: Asset> PartialEq for Handle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<T: Asset> Eq for Handle<T> {}

impl<T: Asset> PartialOrd for Handle<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.id.cmp(&other.id))
    }
}

impl<T: Asset> Ord for Handle<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.id.cmp(&other.id)
    }
}

impl<T: Asset> Default for Handle<T> {
    fn default() -> Self {
        Handle::weak(HandleId::default::<T>())
    }
}
