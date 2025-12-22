/*
use graphics::{
    commands::CommandBuffer,
    types::Bounds,
    widget::{Anchor, Context, DesiredSize, FrameContext, Sender, Widget},
};

#[allow(dead_code, unused_variables)]
pub trait TimerCallback<C: Context>: Callbacks {
    fn on_triggered(&self, sender: &mut Sender<C>) {}
}

#[derive(Default)]
pub struct TimerMock;
impl<C: Context> TimerCallback<C> for TimerMock {}
impl Callbacks for TimerMock {}

#[derive(WidgetQuery)]
pub struct Timer<C, CB = TimerMock, ID = DefaultID>
where
    C: Context,
    CB: TimerCallback<C>,
    ID: WidgetID,
{
    pub interval: f64,
    pub running: bool,
    pub repeat: bool,

    elapsed_time: f64,
    id: ID::IdType,
    callbacks: CB,
    _phantom: std::marker::PhantomData<C>,
}

impl<C, CB> Timer<C, CB, NoID>
where
    C: Context,
    CB: TimerCallback<C>,
{
    #[must_use]
    pub fn new() -> Self {
        Self::new_with_id(())
    }
}

impl<C, CB> Timer<C, CB, StaticID>
where
    C: Context,
    CB: TimerCallback<C>,
{
    #[must_use]
    pub fn new(id: &'static str) -> Self {
        Self::new_with_id(id)
    }
}

impl<C, CB> Timer<C, CB, DefaultID>
where
    C: Context,
    CB: TimerCallback<C>,
{
    #[must_use]
    pub fn new() -> Self {
        Self::new_with_id(None)
    }

    #[must_use]
    pub fn with_id(id: impl Into<String>) -> Self {
        Self::new_with_id(Some(id.into()))
    }
}

impl<C, CB, ID> Default for Timer<C, CB, ID>
where
    C: Context,
    CB: TimerCallback<C>,
    ID: WidgetID,
{
    fn default() -> Self {
        Self::new_with_id(ID::IdType::default())
    }
}

impl<C, CB, ID> Timer<C, CB, ID>
where
    C: Context,
    CB: TimerCallback<C>,
    ID: WidgetID,
{
    fn new_with_id(id: ID::IdType) -> Self {
        Self {
            interval: 0.0,
            running: false,
            repeat: false,
            elapsed_time: f64::MAX,
            id,
            callbacks: CB::default(),
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<C, CB, ID> Widget<C> for Timer<C, CB, ID>
where
    C: Context,
    CB: TimerCallback<C>,
    ID: WidgetID,
{
    fn desired_size(&self) -> DesiredSize {
        DesiredSize::Ignore
    }

    fn draw<'frame>(&'frame self, _: &mut CommandBuffer<'frame>) {}

    fn layout(&mut self, _: Bounds) {}

    fn update(&mut self, ctx: &FrameContext, sender: &mut Sender<C>) {
        if !self.running {
            return;
        }

        if self.elapsed_time < self.interval {
            self.elapsed_time += ctx.delta_time();
            return;
        }

        self.elapsed_time = 0.0;
        self.callbacks.on_triggered(sender);

        if !self.repeat {
            self.running = false;
        }
    }

    fn anchor(&self) -> Anchor {
        Anchor::Left
    }
}
*/
