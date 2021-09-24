use util::cons::ConsFlatten;

use crate::{CommandBuffer, World};

use super::schedule::Runnable;

pub trait QuerySet: Send + Sync {
    fn filter_components(&mut self, world: &World);
}
impl QuerySet for () {
    fn filter_components(&mut self, _: &World) {}
}

pub trait SystemFn<Q> {
    fn run(&mut self, commands: &mut CommandBuffer, queries: &mut Q);
}

impl<F, Q> SystemFn<Q> for F
where
    Q: QuerySet,
    F: FnMut(&mut CommandBuffer, &mut Q),
{
    fn run(&mut self, commands: &mut CommandBuffer, queries: &mut Q) {
        (self)(commands, queries);
    }
}

pub struct System<Q, F> {
    queries: Q,
    run_fn: F,
    command_buffer: Option<CommandBuffer>,
}

impl<Q, F> Runnable for System<Q, F>
where
    Q: QuerySet,
    F: SystemFn<Q>,
{
    fn command_buffer_mut(&mut self) -> Option<&mut CommandBuffer> {
        self.command_buffer.as_mut()
    }

    unsafe fn run_unsafe(&mut self, _world: &crate::World) {
        let queries = &mut self.queries;
        let command = self.command_buffer.get_or_insert(CommandBuffer::new());

        let borrow_fn = &mut self.run_fn;
        borrow_fn.run(command, queries);
    }
}

#[derive(Default)]
pub struct SystemBuilder<Q = ()> {
    queries: Q,
}

impl SystemBuilder<()> {
    pub fn new() -> Self {
        SystemBuilder::default()
    }
}

impl<Q> SystemBuilder<Q>
where
    Q: 'static + Send + ConsFlatten,
{
    pub fn build<F>(self, run_fn: F) -> System<Q, F>
    where
        F: FnMut(&mut CommandBuffer, &mut Q),
    {
        System {
            queries: self.queries,
            run_fn,
            command_buffer: None,
        }
    }
}
