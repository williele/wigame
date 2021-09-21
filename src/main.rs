use std::{sync::Arc, thread, time::Duration};

use app::{Components, Entity, Query};
use rayon::prelude::*;

#[derive(Debug)]
struct Foo(i32);

#[derive(Debug)]
struct Bar(i32);

#[derive(Debug, Default)]
struct Entities {
    counter: u32,
}
impl Entities {
    pub fn alloc(&mut self) -> Entity {
        self.counter += 1;
        Entity::new(self.counter - 1, 0)
    }
}

fn main() {
    let mut entities = Entities::default();
    let mut components = Components::default();
    let a = entities.alloc();
    let b = entities.alloc();
    let c = entities.alloc();

    components.insert(a, Foo(0));
    components.insert(a, Bar(0));
    components.insert(b, Foo(1));
    components.insert(c, Foo(2));
    components.insert(c, Bar(2));

    let ents = Query::new::<Foo>(&components).or_with::<Bar>().vec();
    let components = Arc::new(components);

    let c = components.clone();
    ents.par_iter().for_each(move |ent| {
        c.get_unchecked::<Foo>(ent.clone()).write().0 += 1;
        if let Some(bar) = c.get::<Bar>(ent.clone()) {
            bar.write().0 += 1;
        }
        thread::sleep(Duration::from_millis(200));
    });

    let c = components.clone();
    ents.par_iter().for_each(move |ent| {
        let foo = c.get_unchecked::<Foo>(ent.clone()).read();
        let bar = c.get::<Bar>(ent.clone()).map(|v| v.read());
        println!("{:?}) {:?} {:?}", ent, foo, bar);
    });
}
