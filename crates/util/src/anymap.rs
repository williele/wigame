use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

#[derive(Debug, Default)]
pub struct AnyMap {
    map: HashMap<TypeId, Box<dyn Any>>,
}

impl AnyMap {
    pub fn get<T: Any>(&self) -> Option<&T> {
        let type_id = TypeId::of::<T>();
        self.map.get(&type_id).and_then(|v| v.downcast_ref::<T>())
    }

    pub fn get_mut<T: Any>(&mut self) -> Option<&mut T> {
        let type_id = TypeId::of::<T>();
        self.map
            .get_mut(&type_id)
            .and_then(|v| v.downcast_mut::<T>())
    }

    pub fn has<T: Any>(&self) -> bool {
        let type_id = TypeId::of::<T>();
        self.map.contains_key(&type_id)
    }

    pub fn insert<T: Any>(&mut self, value: T) -> Option<T> {
        self.map
            .insert(value.type_id(), Box::new(value))
            .and_then(|v| v.downcast::<T>().ok().map(|v| *v))
    }

    pub fn remove<T: Any>(&mut self) -> Option<T> {
        let type_id = TypeId::of::<T>();
        self.map
            .remove(&type_id)
            .and_then(|v| v.downcast::<T>().ok().map(|v| *v))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct Foo {
        a: i32,
    }
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct Bar {
        b: i32,
    }

    #[test]
    fn anymap() {
        let mut m = AnyMap::default();
        assert!(!m.has::<Foo>());
        assert!(m.insert(Foo { a: 10 }).is_none());

        assert!(m.has::<Foo>());
        assert!(!m.has::<Bar>());
        assert!(matches!(m.get::<Foo>(), Some(&Foo { a: 10 })));
        assert!(matches!(m.insert(Foo { a: 12 }), Some(Foo { a: 10 })));
        assert!(m.insert(Bar { b: 2 }).is_none());
        m.get_mut::<Bar>().unwrap().b = 22;
        assert!(matches!(m.get::<Bar>(), Some(&Bar { b: 22 })));
    }
}
