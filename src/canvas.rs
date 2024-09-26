use crate::{canvas};
use eframe::App;
use eframe::egui::{self, CentralPanel, Context};
use eframe::emath::TSTransform;
use eframe::epaint::{Color32, Pos2, Rect, vec2};
use egui::{Response, Ui};
use crate::pan_zoom::{PanZoom};
use crate::traits::View;

#[derive(Default)]
pub struct Canvas{
   pub transform: TSTransform,
}

impl View for Canvas {
     fn ui(&mut self, ui:&mut Ui,pan_zoom: &mut PanZoom)->Response{
         let grid_spacing = 20.0; // 网格间隔20像素
         let grid_color = egui::Color32::from_gray(200); // 浅灰色网格线
         let canvas_area=egui::Area::new(egui::Id::new("my_area")).order(egui::Order::Background);
         let canvas_layer_id=canvas_area.layer();
         // 在这里创建area,area只在 CentralPanel 变换，如果不创建area，ui的变换会影响其他panel
        canvas_area.show(ui.ctx(), |ui| {
                 ui.label("Floating text!");

                 let min = ui.min_rect().min; // 画布区域的最小坐标
                 let max = ui.max_rect().max; // 画布区域的最大坐标


                 for x in (min.x as i32..max.x as i32).step_by(grid_spacing as usize) {

                     let start = egui::pos2(x as f32, min.y);
                     let end = egui::pos2(x as f32, max.y);
                     ui.painter().line_segment([start, end], (1.0, grid_color));
                 }

                 for y in (min.y as i32..max.y as i32).step_by(grid_spacing as usize) {
                     let start = egui::pos2(min.x, y as f32);
                     let end = egui::pos2(max.x, y as f32);
                     ui.painter().line_segment([start, end], (1.0, grid_color));
                 }
             ui.ctx().set_transform_layer(canvas_layer_id, self.transform*pan_zoom.transform);
           //  println!("canvas_layer_id id is {:?},ui id is {:?}",canvas_layer_id,ui.id());
             }).response


    }
}

// pub trait PanZoom1{
//     fn pan_zoom(){
//
//     }
// }