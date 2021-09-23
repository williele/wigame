use app::{Entities, Lock, Read, System, Try, World};
use rayon::prelude::*;

#[derive(Debug)]
struct Foo(i32);
impl Foo {
    fn increment(&mut self) {
        self.0 += 1;
    }
}

#[derive(Debug)]
struct Bar(i32);

struct WriteSystem;

impl System for WriteSystem {
    fn run(&mut self, query: &app::QueryEntry) {
        query
            .filter::<Lock<Foo>>()
            .with::<Bar>()
            .vec()
            .par_iter()
            .for_each(|foo| foo.write().increment());

        query
            .filter::<(Entities, Read<Foo>, Try<Read<Bar>>)>()
            .vec()
            .iter()
            .for_each(|data| println!("{:?}", data));
    }
}

struct ReadSystem;
impl System for ReadSystem {
    fn run(&mut self, query: &app::QueryEntry) {
        query
            .filter::<(Entities, Read<Foo>, Read<Bar>)>()
            .vec()
            .par_iter()
            .for_each(|data| println!("{:?}", data))
    }
}

fn main() {
    let mut world = World::default();
    let a = world.spawn().with(Foo(0)).with(Bar(0)).build();
    let b = world.spawn().with(Foo(1)).with(Bar(1)).build();
    let _c = world.spawn().with(Foo(2)).with(Bar(2)).build();

    world
        .query()
        .filter::<(Entities, Read<Foo>, Read<Bar>)>()
        .vec()
        .iter()
        .for_each(|data| println!("{:?}", data));

    println!("=============================");

    world.remove_commponent::<Bar>(b);
    world
        .query()
        .filter::<(Entities, Read<Foo>, Read<Bar>)>()
        .vec()
        .iter()
        .for_each(|data| println!("{:?}", data));

    println!("=============================");

    world.despawn(a);
    world
        .query()
        .filter::<(Entities, Read<Foo>, Read<Bar>)>()
        .vec()
        .iter()
        .for_each(|data| println!("{:?}", data));

    println!("=============================");

    world.spawn().with(Foo(0)).with(Bar(0)).build();
    world
        .query()
        .filter::<(Entities, Read<Foo>, Read<Bar>)>()
        .vec()
        .iter()
        .for_each(|data| println!("{:?}", data));

    println!("=============================");

    world.add_component(b, Bar(1));
    world
        .query()
        .filter::<(Entities, Read<Foo>, Read<Bar>)>()
        .vec()
        .iter()
        .for_each(|data| println!("{:?}", data));

    // let mut app = App::new();
    // app.add_system(WriteSystem);
    // for i in 0..100 {
    //     if rand::random::<bool>() {
    //         app.spawn().with(Foo(i)).with(Bar(i)).build();
    //     } else {
    //         app.spawn().with(Foo(i)).build();
    //     }
    // }

    // app.update();
}
