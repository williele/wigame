use std::marker::PhantomData;

use util::cons::{ConsAppend, ConsFlatten};

use crate::{
    BoxedStageLabel, CommandBuffer, IntoView, Query, Read, Resource, ResourceSet, StageLabel,
    UnsafeResources, World, Write,
};

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

pub trait SystemFn<R, Q>
where
    R: ResourceSet<'static>,
    Q: QuerySet,
{
    fn run(
        &mut self,
        world: &World,
        commands: &mut CommandBuffer,
        resources: &mut R::Item,
        queries: &mut Q,
    );
}

impl<F, R, Q> SystemFn<R, Q> for F
where
    F: FnMut(&World, &mut CommandBuffer, &mut R::Item, &mut Q),
    R: ResourceSet<'static>,
    Q: QuerySet,
{
    fn run(
        &mut self,
        world: &World,
        commands: &mut CommandBuffer,
        resources: &mut R::Item,
        queries: &mut Q,
    ) {
        (self)(world, commands, resources, queries);
    }
}

struct ResourceMarker<T>(PhantomData<*const T>);
unsafe impl<T: Send> Send for ResourceMarker<T> {}
unsafe impl<T: Sync> Sync for ResourceMarker<T> {}

pub struct System<R, Q, F> {
    _resources: ResourceMarker<R>,
    queries: Q,
    stage: Option<BoxedStageLabel>,
    run_fn: F,
    command_buffer: Option<CommandBuffer>,
}

impl<R, Q, F> Runnable for System<R, Q, F>
where
    R: for<'a> ResourceSet<'a>,
    Q: QuerySet,
    F: SystemFn<R, Q>,
{
    fn command_buffer_mut(&mut self) -> Option<&mut CommandBuffer> {
        self.command_buffer.as_mut()
    }

    fn stage(&self) -> Option<BoxedStageLabel> {
        self.stage.clone()
    }

    unsafe fn run_unsafe(&mut self, world: &World, resources: &UnsafeResources) {
        let resources_static = &*(resources as *const UnsafeResources);
        let mut resources = R::fetch_unchecked(resources_static);

        let queries = &mut self.queries;
        let command = self.command_buffer.get_or_insert(CommandBuffer::new());

        let borrow_fn = &mut self.run_fn;
        borrow_fn.run(world, command, &mut resources, queries);
    }
}

pub struct SystemBuilder<R = (), Q = ()> {
    queries: Q,
    resources: R,
    stage: Option<BoxedStageLabel>,
}

impl SystemBuilder<(), ()> {
    pub fn new() -> Self {
        SystemBuilder {
            queries: (),
            resources: (),
            stage: None,
        }
    }
}

impl<R, Q> SystemBuilder<R, Q>
where
    R: 'static + ConsFlatten,
    Q: 'static + Send + ConsFlatten,
{
    pub fn on_stage<L>(self, label: L) -> SystemBuilder<R, Q>
    where
        L: StageLabel,
    {
        SystemBuilder {
            queries: self.queries,
            resources: self.resources,
            stage: Some(label.dyn_clone()),
        }
    }

    pub fn read_resource<T>(self) -> SystemBuilder<<R as ConsAppend<Read<T>>>::Output, Q>
    where
        T: 'static + Resource,
        R: ConsAppend<Read<T>>,
        <R as ConsAppend<Read<T>>>::Output: ConsFlatten,
    {
        SystemBuilder {
            queries: self.queries,
            resources: ConsAppend::append(self.resources, Read::<T>::default()),
            stage: self.stage,
        }
    }

    pub fn write_resource<T>(self) -> SystemBuilder<<R as ConsAppend<Write<T>>>::Output, Q>
    where
        T: 'static + Resource,
        R: ConsAppend<Write<T>>,
        <R as ConsAppend<Write<T>>>::Output: ConsFlatten,
    {
        SystemBuilder {
            queries: self.queries,
            resources: ConsAppend::append(self.resources, Write::<T>::default()),
            stage: self.stage,
        }
    }

    pub fn with_query<V>(
        self,
        query: Query<V>,
    ) -> SystemBuilder<R, <Q as ConsAppend<Query<V>>>::Output>
    where
        V: IntoView,
        Q: ConsAppend<Query<V>>,
    {
        SystemBuilder {
            queries: ConsAppend::append(self.queries, query),
            resources: self.resources,
            stage: self.stage,
        }
    }

    pub fn build<F>(
        self,
        run_fn: F,
    ) -> System<<R as ConsFlatten>::Output, <Q as ConsFlatten>::Output, F>
    where
        F: FnMut(
            &World,
            &mut CommandBuffer,
            &mut <<R as ConsFlatten>::Output as ResourceSet<'static>>::Item,
            &mut <Q as ConsFlatten>::Output,
        ),
        <R as ConsFlatten>::Output: for<'a> ResourceSet<'a>,
        <Q as ConsFlatten>::Output: QuerySet,
    {
        System {
            _resources: ResourceMarker(PhantomData),
            queries: self.queries.flatten(),
            stage: self.stage,
            run_fn,
            command_buffer: None,
        }
    }
}
