use std::cell::UnsafeCell;

use crate::{CommandBuffer, World};

pub trait ParRunnable: Runnable + Send + Sync {}
impl<T: Runnable + Send + Sync> ParRunnable for T {}

pub trait Runnable {
    unsafe fn run_unsafe(&mut self, world: &World);

    fn command_buffer_mut(&mut self) -> Option<&mut CommandBuffer>;

    fn run(&mut self, world: &World) {
        unsafe {
            self.run_unsafe(world);
        }
    }
}

struct SystemBox(UnsafeCell<Box<dyn ParRunnable>>);
unsafe impl Send for SystemBox {}
unsafe impl Sync for SystemBox {}

impl SystemBox {
    // unsafe fn get(&self) -> &dyn ParRunnable {
    //     std::ops::Deref::deref(&*self.0.get())
    // }

    unsafe fn get_mut(&self) -> &mut dyn ParRunnable {
        std::ops::DerefMut::deref_mut(&mut *self.0.get())
    }
}

pub struct ScheduleExec {
    systems: Vec<SystemBox>,
}

impl ScheduleExec {
    pub fn new(systems: Vec<Box<dyn ParRunnable>>) -> Self {
        Self {
            systems: systems
                .into_iter()
                .map(|s| SystemBox(UnsafeCell::new(s)))
                .collect(),
        }
    }

    pub fn execute(&mut self, world: &mut World) {
        self.run_systems(world);
        self.flush_command_buffers(world);
    }

    pub fn run_systems(&mut self, world: &mut World) {
        self.systems.iter_mut().for_each(|system| {
            let system = unsafe { system.get_mut() };
            unsafe { system.run_unsafe(world) };
        });
    }

    pub fn flush_command_buffers(&mut self, world: &mut World) {
        self.systems.iter().for_each(|system| {
            let system = unsafe { system.get_mut() };
            if let Some(cmd) = system.command_buffer_mut() {
                cmd.flush(world);
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use crate::system::system::SystemBuilder;
    use crate::Query;

    use super::*;

    #[derive(Debug)]
    struct Foo(i32);
    #[derive(Debug)]
    struct Bar(i32);

    fn foo_system() -> impl ParRunnable {
        SystemBuilder::new()
            .with_query(Query::<(&mut Foo, &Bar)>::new())
            .with_query(Query::<&Bar>::new())
            .build(|world, _, (foos, bars)| {
                println!("----- foo");
                for (foo, bar) in foos.iter(world) {
                    foo.0 += 1 + bar.0;
                }

                for bar in bars.iter(world) {
                    println!("{:?}", bar);
                }
            })
    }

    fn bar_system() -> impl ParRunnable {
        SystemBuilder::new()
            .with_query(Query::<(&Foo, Option<&Bar>)>::new())
            .build(|world, _, bars| {
                println!("----- bar");
                for data in bars.iter(world) {
                    println!("{:?}", data)
                }
            })
    }

    #[test]
    fn schedule_exec() {
        let mut world = World::default();
        world.spawn().with(Foo(0)).with(Bar(0)).build();
        world.spawn().with(Foo(1)).build();
        world.spawn().with(Foo(2)).with(Bar(2)).build();
        world.flush();

        let mut exec = ScheduleExec::new(vec![Box::new(foo_system()), Box::new(bar_system())]);
        exec.execute(&mut world);
    }
}
