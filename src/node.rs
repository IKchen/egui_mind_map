use egui::{Align2, FontId, Label};
use crate::node_graph::{GraphState,NodeId};
use eframe::egui::{self, CentralPanel, Context};
use eframe::emath::TSTransform;
use eframe::epaint::{CircleShape, Color32, FontFamily, Pos2, Rect, vec2};
use egui::{Area, Button, FontDefinitions, Id, Key, LayerId, Order, Response, Rounding, Sense, Stroke, Style, Ui, Vec2, Widget};
use rand::{random, Rng};
use rand::prelude::*;
use crate::pan_zoom::PanZoom;
use crate::traits::View;

#[derive(Debug)]
pub enum NodeState{
    Editing,
    Selected,
    UnSelected,
    Hover,

}
pub enum NodeResponse{
    AddNode(NodeId),
    DeleteNode(NodeId),
    Selected(NodeId),
    UnSelected(NodeId),
    EditNode(NodeId),
    None,
}
#[derive(Debug)]
pub struct Node{
    pub node_id:NodeId,
    pub transform: TSTransform,
    pub node_pos: Pos2,
    pub node_size:Vec2,
    pub node_color:Color32,
    pub node_text:String,
  //  pub node_state: NodeState,
    pub father_id:Option<NodeId>,
    pub button_pos:Pos2,// 确定 展开按钮的位置
    //这里绘制矩形 用area不合适，应该ui 根据rect 大小分配一个 response?
  //  pub node_area: Area //用来绘制 node
}
impl Default for Node {
    fn default() -> Self {
        let mut rng = rand::thread_rng();

        // 生成一个在 0 到 255 之间的随机数
        let random_number: u8 = rng.gen_range(0..=255);
      //  println!("the random is {:?}",random_number);
        Self {
            node_id:NodeId::default(),
            transform: TSTransform::default(),
            node_pos: Pos2::new(100.0, 200.0),
            node_size: Vec2::new(100.0, 50.0),
            node_text : String::from("text"),
            node_color:Color32::from_rgb(150, 150, 250),
            father_id:None,
            button_pos:Pos2::new(100.0, 200.0) + Vec2::new(100.0, 0.0) / 2.0 + Vec2::new(10.0, 0.0)

        }
    }
}
impl  Node{
    //设置节点颜色
    pub fn set_color(&mut self){
        // 创建一个随机数生成器
        let mut rng = rand::thread_rng();

        // 生成一个在 0 到 255 之间的随机数
        let random_number: u8 = rng.gen_range(0..=255);
        self.node_color=Color32::from_rgb(random_number,random_number,random_number);
    }
    //设置节点位置
    pub fn set_pos(&mut self,pos:Pos2){
        self.node_pos=pos;
    }


}

impl View for Node {
    fn draw(& mut self, ui: &mut Ui, pan_zoom: &mut PanZoom, node_state: &mut NodeState) ->NodeResponse {
        //这里每次绘制要用新的 transform pos 和size ，不能用 self 的pos 和size ，因为每次循环累计缩放和平移
        let mut transformed_pos = pan_zoom.transform* self.node_pos;
        let transformed_size = pan_zoom.transform.scaling * self.node_size;
        //self.node_pos=pan_zoom.transform* self.node_pos;

        let rect = egui::Rect::from_center_size(transformed_pos, transformed_size);
       // let node_area=Area::new(Id::new(self.node_id)).order(Order::Middle).default_pos(rect.min).default_size(transformed_size).movable(true);//这里要设置每个node area的 大小 和位置

        // 不绘制矩形，只绘制编辑框
        match node_state {
            NodeState::Editing => {
                // 编辑状态下，绘制编辑框
                let mut text_zone_style = Style::default(); // 编辑框样式
                text_zone_style.visuals.override_text_color = Some(egui::Color32::from_rgb(0, 255, 0)); // 字体颜色为绿色
                text_zone_style.visuals.extreme_bg_color = egui::Color32::from_rgb(123, 123, 0); // 背景颜色

                ui.set_style(text_zone_style);
                //put 来设置 ui 绘制的位置和矩形大小
                let text_response = ui.put(
                        rect,
                        egui::TextEdit::multiline(&mut self.node_text).desired_rows(1).desired_width(self.node_size.x - 10.0),
                    );

                // 当编辑框失去焦点时，退出编辑状态
                return if text_response.lost_focus() {
                    NodeResponse::UnSelected(self.node_id) // 返回对应响应
                } else {
                    NodeResponse::EditNode(self.node_id)
                };
            }
            _ => {
                // 非编辑状态时绘制矩形和文本
                let response = ui.allocate_rect(rect, Sense::click_and_drag());

                if response.dragged() {
                    let delta = response.drag_delta();
                    transformed_pos += delta;
                    self.node_pos += delta; // 同步更新节点位置
                    self.button_pos+=delta;
                }

                ui.painter().rect_filled(rect, 5.0, self.node_color);

                ui.painter().text(
                    rect.center(),                     // 矩形的中心位置
                    Align2::CENTER_CENTER,             // 文本对齐方式，设置为中心对齐
                    &self.node_text,                   // 文本内容
                    FontId::default(),                 // 使用默认字体
                    Color32::BLACK,                    // 文本颜色
                );

                // 处理状态切换
                match node_state {
                    NodeState::Hover => {
                        self.node_color = Color32::from_rgb(200, 150, 250);

                        if response.double_clicked() {

                            return NodeResponse::EditNode(self.node_id);
                        }

                        if response.clicked() {
                            return NodeResponse::Selected(self.node_id);
                        }
                    }
                    NodeState::Selected => {
                        self.node_color = Color32::from_rgb(250, 0, 0);
                        if response.clicked() {
                            return NodeResponse::UnSelected(self.node_id);
                        }
                        if ui.ctx().input(|x| {x.key_pressed(Key::Tab)}){
                            return  NodeResponse::AddNode(self.node_id)
                        }
                    }
                    NodeState::UnSelected => {
                        self.node_color = Color32::from_rgb(150, 150, 250);
                        if response.hovered() {
                            *node_state = NodeState::Hover;
                        }
                    }
                    _ => {}
                }
            }
        }

        NodeResponse::None
    }
    fn draw_button(& mut self, ui: &mut Ui, pan_zoom: &mut PanZoom, button_state:&mut ButtonState)  {
        //这里每次绘制要用新的 transform pos 和size ，不能用 self 的pos 和size ，因为每次循环累计缩放和平移
        let transformed_button_size = pan_zoom.transform.scaling * Vec2::new(5.0, 5.0);//5是半径
        let transformed_node_size=pan_zoom.transform.scaling * self.node_size;
        let mut transformed_node_pos = pan_zoom.transform* self.node_pos;
        let transformed_button_pos = pan_zoom.transform* self.button_pos;
        let rect = Rect::from_center_size(transformed_button_pos, transformed_button_size);

        ui.painter().circle_filled(transformed_button_pos, pan_zoom.transform.scaling*5.0, self.node_color);
        //button 与 node 的连线
        ui.painter().line_segment(
            [transformed_node_pos+transformed_node_size*Vec2::new(0.5,0.0)  , transformed_button_pos-transformed_button_size*Vec2::new(0.5,0.0)],
            (2.0, Color32::from_rgb(255,0,0))
        );

            match button_state {
                ButtonState::UnFold => {
                    ui.painter().line_segment(
                        [transformed_button_pos + transformed_button_size*Vec2::new(0.5,0.0),transformed_button_pos - transformed_button_size*Vec2::new(0.5,0.0)],
                        (2.0, Color32::from_white_alpha(255)),
                    );
                }
                ButtonState::Fold => {
                    ui.painter().line_segment(
                        [transformed_button_pos + transformed_button_size*Vec2::new(0.0,0.5), transformed_button_pos- transformed_button_size*Vec2::new(0.0,0.5)],
                        (2.0, Color32::from_white_alpha(255)),
                    );
                    ui.painter().line_segment(
                        [transformed_button_pos +transformed_button_size*Vec2::new(0.5,0.0), transformed_button_pos -transformed_button_size*Vec2::new(0.5,0.0)],
                        (2.0, Color32::from_white_alpha(255)),
                    );
                }

            }

            let response = ui.allocate_rect(rect, egui::Sense::click_and_drag());
            if response.clicked() {
                *button_state = match button_state {
                    ButtonState::UnFold =>   ButtonState::Fold,
                    ButtonState::Fold =>  ButtonState::UnFold,
                };
                ui.ctx().request_repaint(); // 强制重绘UI
               // println!("the button state is {:?}", button_state);
            }
    }
}
#[derive(Copy,Clone,Debug)]
pub enum ButtonState{
   // Hove,
    Fold,
    UnFold
}
