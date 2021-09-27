use crate::{Events, ParRunnable, Resource, Resources, Schedule, Stage, StageLabel, World};

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum AppStage {
    Begin,
    Startup,
    PreUpdate,
    Update,
    PostUpdate,
    End,
}
impl StageLabel for AppStage {
    fn dyn_clone(&self) -> Box<dyn StageLabel> {
        match self {
            AppStage::Begin => Box::new("App:Begin"),
            AppStage::Startup => Box::new("App:Startup"),
            AppStage::PreUpdate => Box::new("App:PreUpdate"),
            AppStage::Update => Box::new("App:Update"),
            AppStage::PostUpdate => Box::new("App:PostUpdate"),
            AppStage::End => Box::new("App:End"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AppExit;

pub trait Plugin {
    fn build(&mut self, app: &mut App);
}

pub struct App {
    pub world: World,
    pub schedule: Schedule,
    pub resources: Resources,
    runner: Box<dyn Fn(App)>,
}

impl Default for App {
    fn default() -> Self {
        App {
            world: Default::default(),
            resources: Default::default(),
            schedule: Default::default(),
            runner: Box::new(run_once),
        }
    }
}

impl App {
    pub fn new() -> Self {
        let mut app = App::default();
        app.add_stage(AppStage::Begin, Stage::sequence())
            .add_stage(AppStage::Startup, Stage::sequence_once())
            .add_stage(AppStage::PreUpdate, Stage::sequence())
            .add_stage(AppStage::Update, Stage::sequence())
            .add_stage(AppStage::PostUpdate, Stage::sequence())
            .add_stage(AppStage::End, Stage::sequence())
            .add_event::<AppExit>();
        app
    }

    pub fn add_plugin<P>(&mut self, mut plugin: P) -> &mut Self
    where
        P: Plugin,
    {
        plugin.build(self);
        self
    }

    pub fn add_resource<R: Resource>(&mut self, resource: R) -> &mut Self {
        self.resources.insert(resource);
        self
    }

    pub fn add_event<T: 'static>(&mut self) -> &mut Self {
        self.add_resource(Events::<T>::default())
            .add_system(Events::<T>::update_sys())
    }

    pub fn add_stage(&mut self, label: impl StageLabel, stage: Stage) -> &mut Self {
        self.schedule.add_stage(label, stage);
        self
    }

    pub fn add_stage_before(
        &mut self,
        target: impl StageLabel,
        label: impl StageLabel,
        stage: Stage,
    ) -> &mut Self {
        self.schedule.add_stage_before(target, label, stage);
        self
    }

    pub fn add_stage_after(
        &mut self,
        target: impl StageLabel,
        label: impl StageLabel,
        stage: Stage,
    ) -> &mut Self {
        self.schedule.add_stage_after(target, label, stage);
        self
    }

    pub fn add_system<S>(&mut self, system: S) -> &mut Self
    where
        S: ParRunnable + 'static,
    {
        if system.stage().is_some() {
            self.schedule.add_system(system);
        } else {
            self.add_system_to_stage(AppStage::Update, system);
        }
        self
    }

    pub fn add_system_to_stage<S>(&mut self, label: impl StageLabel, system: S) -> &mut Self
    where
        S: ParRunnable + 'static,
    {
        self.schedule.add_system_to_stage(label, system);
        self
    }

    pub fn set_runner(&mut self, run_fn: impl Fn(App) + 'static) -> &mut Self {
        self.runner = Box::new(run_fn);
        self
    }

    pub fn update(&mut self) {
        self.schedule.run(&mut self.world, &mut self.resources);
    }

    pub fn run(&mut self) {
        let mut app = std::mem::replace(self, App::default());
        let runner = std::mem::replace(&mut app.runner, Box::new(run_once));
        (runner)(app);
    }
}

fn run_once(mut app: App) {
    app.update();
}
