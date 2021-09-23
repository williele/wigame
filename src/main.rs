use app::{Entity, Read, World};
// use rayon::prelude::*;

#[derive(Debug)]
pub struct Character(i32);

#[derive(Debug)]
pub struct Target(Entity);

fn main() {
    let mut world = World::default();

    let a = world.spawn().with(Character(0)).build();
    world.spawn().with(Character(1)).with(Target(a)).build();

    for (character, target) in world
        .query()
        .filter::<(Read<Character>, Read<Target>)>()
        .all()
    {
        if let Some(target) = world.query().filter::<Read<Character>>().entity(target.0) {
            println!("{:?} target {:?}", character, target)
        }
    }

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
