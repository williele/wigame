use util::cons::{ConsAppend, ConsFlatten};

use crate::{CommandBuffer, IntoView, Query, World};

use super::executor::Runnable;

pub trait QuerySet: Send + Sync {}

macro_rules! impl_queryset_tuple {
    ($($name: ident),*) => {
        impl<$($name,)*> QuerySet for ($($name,)*)
        where
            $($name: QuerySet,)*
        {}
    };
}

macro_rules! queryset_tuple {
    ($head_ty:ident) => {
        impl_queryset_tuple!($head_ty);
    };
    ($head_ty:ident, $( $tail_ty:ident ),*) => (
        impl_queryset_tuple!($head_ty, $($tail_ty),*);
        queryset_tuple!($($tail_ty),*);
    );
}

queryset_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z);

impl QuerySet for () {}
impl<V> QuerySet for Query<V> where V: IntoView + Send + Sync {}

pub trait SystemFn<Q> {
    fn run(&mut self, world: &World, commands: &mut CommandBuffer, queries: &mut Q);
}

impl<F, Q> SystemFn<Q> for F
where
    Q: QuerySet,
    F: FnMut(&World, &mut CommandBuffer, &mut Q),
{
    fn run(&mut self, world: &World, commands: &mut CommandBuffer, queries: &mut Q) {
        (self)(world, commands, queries);
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

    unsafe fn run_unsafe(&mut self, world: &crate::World) {
        let queries = &mut self.queries;
        let command = self.command_buffer.get_or_insert(CommandBuffer::new());

        let borrow_fn = &mut self.run_fn;
        borrow_fn.run(world, command, queries);
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
    pub fn with_query<V>(
        self,
        query: Query<V>,
    ) -> SystemBuilder<<Q as ConsAppend<Query<V>>>::Output>
    where
        V: IntoView,
        Q: ConsAppend<Query<V>>,
    {
        SystemBuilder {
            queries: ConsAppend::append(self.queries, query),
        }
    }

    pub fn build<F>(self, run_fn: F) -> System<<Q as ConsFlatten>::Output, F>
    where
        F: FnMut(&World, &mut CommandBuffer, &mut <Q as ConsFlatten>::Output),
    {
        System {
            queries: self.queries.flatten(),
            run_fn,
            command_buffer: None,
        }
    }
}