use std::println;
use eframe::emath::{Rect, Vec2};
use eframe::epaint::{Color32, Rounding, Stroke};
use egui::{Pos2, Response, Ui};
use crate::node_graph:: NodeId;
use crate::node::{ButtonState, NodeResponse, NodeState};
use crate::node_graph::{GraphResponse, GraphState};
use crate::pan_zoom::PanZoom;

pub trait NodeGraphs {
    //用draw 函数实现node graph的绘制逻辑
    fn draw(&mut self,  ui:&mut Ui ,pan_zoom: &mut PanZoom,graph_state: &mut GraphState)-> GraphResponse{
      GraphResponse::default()
    }
    fn add_node_with_father_node(&mut self, father_node_id:  NodeId)->NodeId{
        NodeId::default()
    }
    fn add_node_with_position(&mut self, pos2: Pos2)-> NodeId{
        NodeId::default()
    }
    fn select_node(&mut self,node_id:NodeId){}
    fn edite_node(&mut self,node_id:NodeId){}
    fn delete_node(&mut self,node_id:NodeId){}

}
pub trait View {
    //这个函数用来实现node 的绘制
    fn draw(&mut self,  ui:&mut Ui ,pan_zoom: &mut PanZoom,node_state:&mut NodeState)->NodeResponse{
        NodeResponse::None
    }
    //下面这个暂时不用了
    fn ui(& mut self, ui: &mut Ui,pan_zoom: &mut PanZoom) ->Response{
        let (id, rect) = ui.allocate_space(ui.available_size());
        let response = ui.interact(rect, id, egui::Sense::click_and_drag());
        response
    }
    fn draw_button(& mut self, ui: &mut Ui, pan_zoom: &mut PanZoom, button_state:&mut ButtonState){}
}