wit_bindgen::generate!({
    path: "../../../../../specs/plugin-base",
    world: "general",
});

pub struct Example;
impl Guest for Example {
    fn init() {
        panic!("Example plugin failed to initialize");
    }
}

export!(Example);
