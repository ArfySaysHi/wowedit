pub mod app;
pub mod camera;
pub mod camera_controller;
pub mod instance;
pub mod state;
pub mod texture;
pub mod vertex;

// TODO: Add depth_buffer so avoid clipping and layering issues (next tutorial bit)
// TODO: Get a basic ADT rendered as a plane with a heightmap applied, could use an abstract or
// an interface of some kind? Trait? Not sure, but ideally I can just have rendering be a shared
// behaviour for looping through
