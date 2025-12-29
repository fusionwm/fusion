use smithay::output;

pub trait Backend {
    fn create_output(&self) -> output::Output;
    fn mode(&self) -> output::Mode;
}
