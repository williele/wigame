use app::{Entity, Read, System, World};
// use rayon::prelude::*;

#[derive(Debug)]
pub struct Character(i32);

#[derive(Debug)]
pub struct Target(Entity);

struct DemoSystem;
impl System for DemoSystem {
    fn run(&mut self, command: &mut app::CommandBuffer, query: &app::QueryEntry) {
        query
            .filter::<(Read<Character>, Read<Target>)>()
            .all()
            .iter()
            .for_each(|(_, target)| {
                command.despawn(target.0);
            });
    }
}

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
