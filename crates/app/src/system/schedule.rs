use std::collections::HashMap;

use crate::{system::stage::Stage, BoxedStageLabel, ParRunnable, StageLabel, World};

#[derive(Default)]
pub struct Schedule {
    stages: HashMap<BoxedStageLabel, Stage>,
    stage_order: Vec<BoxedStageLabel>,
}

impl Schedule {
    fn get_stage_mut(&mut self, label: &dyn StageLabel) -> Option<&mut Stage> {
        self.stages.get_mut(label.clone())
    }

    fn get_stage_index(&self, target: &dyn StageLabel) -> Option<usize> {
        self.stage_order
            .iter()
            .enumerate()
            .find(|(_i, stage_label)| &***stage_label == target)
            .map(|(i, _)| i)
    }

    pub fn add_stage(&mut self, label: impl StageLabel, stage: Stage) -> &mut Self {
        let label: Box<dyn StageLabel> = Box::new(label);
        self.stage_order.push(label.clone());
        if self.stages.insert(label.clone(), stage).is_some() {
            panic!("Stage already exists: {:?}", label);
        }
        self
    }

    pub fn add_stage_after(
        &mut self,
        target: impl StageLabel,
        label: impl StageLabel,
        stage: Stage,
    ) -> &mut Self {
        let label: Box<dyn StageLabel> = Box::new(label);
        let target_index = self
            .get_stage_index(&target)
            .unwrap_or_else(|| panic!("Target stage does not exist: {:?}.", target));
        self.stage_order.insert(target_index + 1, label.clone());
        if self.stages.insert(label.clone(), stage).is_some() {
            panic!("Stage already exists: {:?}", label);
        }
        self
    }

    pub fn add_stage_before(
        &mut self,
        target: impl StageLabel,
        label: impl StageLabel,
        stage: Stage,
    ) -> &mut Self {
        let label: Box<dyn StageLabel> = Box::new(label);
        let target_index = self
            .get_stage_index(&target)
            .unwrap_or_else(|| panic!("Target stage does not exist: {:?}.", target));
        self.stage_order.insert(target_index, label.clone());
        if self.stages.insert(label.clone(), stage).is_some() {
            panic!("Stage already exists: {:?}", label);
        }
        self
    }

    pub fn add_system_to_stage<S: ParRunnable + 'static>(
        &mut self,
        stage_label: impl StageLabel,
        system: S,
    ) -> &mut Self {
        let stage = self
            .get_stage_mut(&stage_label)
            .unwrap_or_else(move || panic!("Stage '{:?}' does not exist", stage_label));
        stage.add_system(system);
        self
    }

    pub fn run(&mut self, world: &mut World) {
        for label in self.stage_order.iter() {
            let stage = self.stages.get_mut(label).unwrap();
            stage.run(world);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{query::Entities, Query, SystemBuilder};

    use super::*;

    #[derive(Debug)]
    struct Foo(i32);
    #[derive(Debug)]
    struct Bar(i32);

    fn first1() -> impl ParRunnable {
        SystemBuilder::new()
            .with_query(Query::<(&mut Foo, &Bar)>::new())
            .build(|world, _, query| {
                println!("first 1: update foo");
                query
                    .iter(world)
                    .into_iter()
                    .for_each(|(foo, bar)| foo.0 += bar.0);
            })
    }

    fn first2() -> impl ParRunnable {
        SystemBuilder::new()
            .with_query(Query::<(&Foo, &Bar)>::new())
            .build(|world, _, query| {
                println!("first 2: print foo and bar");
                query
                    .iter(world)
                    .into_iter()
                    .for_each(|data| println!("{:?}", data));
            })
    }

    fn second() -> impl ParRunnable {
        SystemBuilder::new()
            .with_query(Query::<(Entities, &Foo, Option<&Bar>)>::new())
            .build(|world, cmd, query| {
                println!("second: despawn none bar");
                query.iter(world).into_iter().for_each(|(ent, _, bar)| {
                    if bar.is_none() {
                        cmd.despawn(ent);
                    }
                });
            })
    }

    fn third() -> impl ParRunnable {
        SystemBuilder::new()
            .with_query(Query::<(&Foo, Option<&Bar>)>::new())
            .build(|world, _, query| {
                println!("third: print foo and bar");
                query
                    .iter(world)
                    .into_iter()
                    .for_each(|data| println!("{:?}", data))
            })
    }

    #[test]
    fn schedule() {
        let mut world = World::default();
        world.spawn().with(Foo(0)).with(Bar(0)).build();
        world.spawn().with(Foo(1)).build();
        world.spawn().with(Foo(2)).with(Bar(2)).build();
        world.flush();

        let mut schedule = Schedule::default();

        schedule.add_stage("first", Stage::sequence());
        schedule.add_stage_after("first", "second", Stage::sequence());
        schedule.add_stage_after("second", "third", Stage::sequence());

        schedule.add_system_to_stage("first", first1());
        schedule.add_system_to_stage("first", first2());
        schedule.add_system_to_stage("second", second());
        schedule.add_system_to_stage("third", third());

        schedule.run(&mut world);
    }
}
