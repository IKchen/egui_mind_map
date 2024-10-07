mod canvas;
mod node;
mod pan_zoom;
mod node_graph;
mod traits;
mod handlers;

use std::thread;
use std::thread::spawn;
use crate::pan_zoom::{PanZoom};
use egui::{Align2, Id, Layout, menu, Order, Painter, Rounding, Sense, TextStyle, TopBottomPanel, Widget, Window, FontData, FontFamily};
use egui::{Ui, Response, Vec2, pos2, Color32};
use crate::handlers::*;
/// 自定义按钮组件
fn custom_button(ui: &mut Ui, text: &str) -> Response {
    let desired_size = Vec2::new(100.0, 30.0); // 按钮大小
    let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click());
    if ui.is_rect_visible(rect) {
        // 绘制按钮
        let visuals = ui.style().interact(&response);
        ui.painter().rect_filled(rect, visuals.rounding, visuals.bg_fill);
        ui.painter().text(
            pos2(rect.center().x, rect.center().y),
            egui::Align2::CENTER_CENTER,
            text,
            egui::TextStyle::resolve(&TextStyle::Small, &Default::default()),
            visuals.text_color(),
        );
    }
    response
}

fn circle_interactable(ui: &mut egui::Ui, center: &mut Pos2, radius: f32, id: Id) -> Response {
    // 计算圆覆盖的矩形区域
    let rect = egui::Rect::from_center_size(*center, vec2(radius * 2.0, radius * 2.0));

    // 使用一个透明按钮覆盖在圆上
    let response = ui.allocate_rect(rect, Sense::click_and_drag()).interact(Sense::click_and_drag());

    if response.dragged() {
        // 如果按钮被拖动，更新圆心位置
        *center += response.drag_delta();
    }

    // 绘制圆
    ui.painter().circle_filled(*center, radius, Color32::from_rgb(150, 150, 250));
// 绘制框
    ui.painter().rect_stroke(rect,Rounding::default(), ui.ctx().style().visuals.window_stroke);
    response
}
use eframe::App;
use eframe::egui::{self, CentralPanel, Context};
use eframe::emath::TSTransform;
use eframe::epaint::{Pos2, Rect, Stroke, vec2};
use eframe::epaint::text::FontDefinitions;
use egui::epaint::text::layout;
use egui::Order::Foreground;
use crate::canvas::{Canvas};
use crate::node::{ButtonState, Node, NodeState};
use crate::node_graph::{GraphResponse, GraphState, NodeGraph, NodeId};
use crate::traits::{NodeGraphs, View};

pub struct MyApp{
    canvas: Canvas,
    node_graph: NodeGraph,
    circle_center:Pos2,
    pan_zoom: PanZoom,
    window_state:bool,
    graph_state: GraphState,
    graph_response: GraphResponse,
}
impl Default for MyApp{
    fn default() -> Self {
        Self{
            canvas:Canvas::default(),
            node_graph:NodeGraph::default(),
            circle_center:Pos2::default(),
            pan_zoom:PanZoom::default(),
            window_state:false,
            graph_state:GraphState::default(),
            graph_response:GraphResponse::default()
        }
    }
}


impl  MyApp{

    //整个应用的缩放和平移
    pub fn pan_zoom( &mut self,ui: &mut Ui,){
        let (id, rect) = ui.allocate_space(ui.available_size());
        let mut response = ui.interact(rect, id, egui::Sense::click_and_drag());


        ui.painter().rect_stroke(rect,Rounding::default(), ui.ctx().style().visuals.window_stroke);

        if response.dragged() {
            self.pan_zoom.transform.translation += response.drag_delta();

        }

        if let Some(pointer)=ui.ctx().input(|i| { i.pointer.hover_pos()})
        {
          //  println!("hover is {:?}",pointer);
            if response.hovered() {
                let pointer_in_layer = self.pan_zoom.transform.inverse()*pointer;
                let zoom_delta = ui.ctx().input(|i| i.zoom_delta());
                let pan_delta = ui.ctx().input(|i| i.smooth_scroll_delta);
                //println!("zoom is {zoom_delta:?}");

// Zoom in on pointer:
                self.pan_zoom.transform = self.pan_zoom.transform
                    * TSTransform::from_translation(pointer_in_layer.to_vec2())
                    * TSTransform::from_scaling(zoom_delta)
                    * TSTransform::from_translation(-pointer_in_layer.to_vec2());


            }
        }
        if response.double_clicked(){

            let Some(pos)= ui.ctx().pointer_hover_pos() else { todo!() };
            //这里要 inverse transform  因为node draw时 要用加 pan ，这里的pos 要还原成 原始左边，不然绘制会加2遍 transform
            self.add_node(Some(self.pan_zoom.transform.inverse()*pos ),None);
           //   println!("double_clicked is {:?}",pos);
        }

    }

    pub fn add_node(&mut self, pos2:Option<Pos2>,father_node_id:Option<NodeId>){
            match pos2 {
                None => {}
                Some(pos) => {
                   let mut node_id= self.node_graph.add_node_with_position(pos);
                    //这里创建完node 之后，还要创建node state
                    self.graph_state.node_state.insert_with_key(|mut node_id: NodeId|{
                        node_id=node_id;
                        NodeState::UnSelected
                    });
                    self.graph_state.graph_button_state.insert_with_key(|mut k| {
                        k = node_id;
                        ButtonState::UnFold
                    }); // 插入新的节点状态
                }
            }
            match father_node_id {
                None => {}
                Some(id) => {
                    let mut node_id=self.node_graph.add_node_with_father_node(id);
                    //这里创建完node 之后，还要创建node state
                    self.graph_state.node_state.insert_with_key(|mut node_id: NodeId|{
                        node_id=node_id;
                        NodeState::UnSelected
                    });
                    self.graph_state.graph_button_state.insert_with_key(|mut k| {
                        k = node_id;
                        ButtonState::UnFold
                    }); // 插入新的节点状态
                }
            }
    }
}

impl App for MyApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {


        egui::TopBottomPanel::top("my_top_panel").show(ctx, |ui| {
            menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("new file").clicked() {
                        self.window_state=true;
                    }

                });
            }).response.layer_id.order=Foreground;

        //    println!("toplayer  id is {:?}", ui.id());//ui 的id 是不一样的，layer的id是一样的
        });
        let central_response=CentralPanel::default().show(ctx, |ui| {
            set_font(ui);

         //   println!("centrallayer  id is {:?}", ui.layer_id());
            let percent=self.pan_zoom.transform.scaling;
            let movement=self.pan_zoom.transform.translation;
            let pointer=ui.ctx().pointer_hover_pos().unwrap_or(Pos2::new(0.0,0.0));

            ui.label(format!("缩放比例 '{percent}', 平移转换 {movement},悬停坐标{pointer}"));
            self.pan_zoom(ui);
         //   let response=self.canvas.ui( ui,& mut self.pan_zoom);
            match self.window_state {
                true => {
                    Window::new("My Window").open(&mut self.window_state).pivot(Align2::CENTER_CENTER).default_pos(ui.max_rect().center()).resize(|r| r.resizable(true)).show(ctx, |ui| {
                        ui.label("Hello World!");
                    });
                }
                false => {}
            }

            let graph_response=self.node_graph.draw(ui,&mut self.pan_zoom,&mut self.graph_state);
            handle_graph_response(&mut self.node_graph,&mut self.graph_state,graph_response);//处理response

        });
        egui::TopBottomPanel::bottom("my_bottom_panel").show(ctx, |ui| {
          //  println!("bottomlayer  id is {:?}", ui.id());
            ui.label("this is a bottom panel!");
        });
    }

}

fn main() {
    let options = eframe::NativeOptions::default();
    let mut myapp=MyApp::default();
 //   handle_state(&mut myapp.graph_state,myapp.graph_response);
    eframe::run_native(
        "Infinite Zoom Canvas",
        options,
        Box::new(|_cc| Ok(Box::new(myapp))),
    ).expect("TODO: panic message");
}

//设置字体
fn set_font( ui:&mut Ui){
    //设置字体
    let mut fonts=FontDefinitions::default();
    fonts.font_data.insert("my_font".to_owned(),
                           FontData::from_static(include_bytes!("../font/SourceHanSerifCN-Bold.ttf")));
    fonts.families.get_mut(&FontFamily::Proportional).unwrap()
        .insert(0, "my_font".to_owned());
    ui.ctx().set_fonts(fonts);
}

