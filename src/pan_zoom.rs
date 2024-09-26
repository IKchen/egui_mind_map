use eframe::emath::{Pos2, TSTransform, vec2};

#[derive(Debug)]
pub struct PanZoom{
   pub transform: TSTransform,
}
impl PanZoom{
    pub fn new() -> Self {
        PanZoom {
            transform: TSTransform::default(),
        }
    }

}
impl Default for PanZoom{
    fn default() -> Self {
        PanZoom {
            transform: TSTransform::default(),
        }
    }
}
