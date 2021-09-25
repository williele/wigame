use app::{Query, World};
use rayon::prelude::*;

#[derive(Debug)]
struct Foo(i32);

#[derive(Debug)]
struct Bar(i32);

#[derive(Debug)]
struct Baz;

fn main() {
    let mut world = World::default();

    world.spawn().with(Foo(0)).with(Bar(10)).build();
    world.spawn().with(Foo(1)).build();
    world.spawn().with(Foo(2)).with(Bar(12)).with(Baz).build();
    world.flush();

    Query::<(&mut Foo, &Bar)>::new()
        .iter(&world)
        .into_par_iter()
        .for_each(|(foo, bar)| foo.0 += bar.0);

    Query::<(&Foo, Option<&Bar>)>::new()
        .iter(&world)
        .into_par_iter()
        .for_each(|data| println!("{:?}", data));
}
