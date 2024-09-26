use std::cmp::PartialEq;
use std::option::Option;
use std::{fmt, thread};
use eframe::emath::Vec2;
use eframe::epaint::{Pos2, pos2, Rounding};
use egui::{Area, Color32, Id, Shape, Stroke, Style, Ui};
use egui::emath::TSTransform;
use egui::epaint::CubicBezierShape;
use slotmap::SlotMap;
use crate::node::{ButtonState, Node, NodeResponse, NodeState};
use crate::pan_zoom::PanZoom;
use crate::traits::{NodeGraphs, View};


slotmap::new_key_type! {
    pub struct NodeId;
}

//node_id 要作为 area id的唯一值，需要实现display trait
impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "node_{:?}", self.0)
    }
}
pub struct GraphResponse{
    pub nodes_response:SlotMap<NodeId, NodeResponse>
}
impl Default for GraphResponse{
    fn default() -> Self {
        Self{nodes_response:SlotMap::default()}
    }
}
pub struct NodeGraph {
    pub nodes: SlotMap<NodeId, Node>,
    pub last_key:Option<NodeId>
}
pub struct GraphState{
    pub node_state: SlotMap<NodeId,NodeState>,
    pub graph_button_state:SlotMap<NodeId,ButtonState>
}
impl GraphState{
    pub fn state_handle(&mut self, graph_response: GraphResponse){

            for (nodeid,response) in graph_response.nodes_response{
                match response {
                    NodeResponse::EditNode(id) => {
                        self.node_state[id]=NodeState::Editing;
                    }
                    NodeResponse::AddNode(id)=> {

                    }
                    NodeResponse::Selected (id)=> {
                        self.node_state[id]=NodeState::Selected;
                    }
                    NodeResponse::UnSelected(id) => {
                        self.node_state[id]=NodeState::UnSelected;
                    }

                    _ => {}
                }
            }

    }
}
impl Default for GraphState {
    fn default() -> Self {
        Self{
            node_state:SlotMap::default(),
            graph_button_state:SlotMap::default()}
    }
}
impl Default for NodeGraph{
   fn default()->Self{
       Self{
           nodes: SlotMap::default(),//default 的值只能用于初始化，不能访问
           last_key:None
       }
    }
}
impl NodeGraph{

    pub fn draw_curve_line(&mut self, ui: &mut Ui, pan_zoom: &PanZoom) {
        for node in self.nodes.values() {

            let father_id=node.father_id;
            match father_id {
                None => {}//没有节点时，不做任何事情
                Some(father_id) => {
                    let father_node=self.nodes.get(father_id).expect("father id get wrong");
                    let control_points = [
                        node.node_pos,
                        pos2((node.node_pos.x + father_node.button_pos.x) / 2.0, node.button_pos.y),
                        pos2((node.node_pos.x + father_node.button_pos.x) / 2.0, father_node.button_pos.y),
                        father_node.button_pos+  Vec2::new(5.0, 5.0)*Vec2::new(0.5,0.0),
                    ];

                    // 应用变换到控制点
                    let transformed_points: [Pos2; 4] = control_points.map(|p| pan_zoom.transform.mul_pos(p));

                    let stroke = Stroke::new(2.0, Color32::from_rgb(255, 0, 0));
                    let color = Color32::from_rgba_premultiplied(0, 0, 0, 0);
                    let shape = draw_bezier_line(stroke, color, transformed_points);

                    ui.painter().add(shape);
                }
            }

        }
    }
}



impl NodeGraphs for NodeGraph{
    fn draw(&mut self, ui: &mut Ui, pan_zoom: &mut PanZoom, graph_state: &mut GraphState) -> GraphResponse {
        let mut graph_response = GraphResponse::default();
        let mut nodes_to_remove = Vec::new();//要删除的节点
        let mut nodes_to_add = Vec::new();//需要新增的节点
        let mut nodes_have_children=Vec::new();//有子节点的 节点，用于绘制展开和折叠按钮
        ui.ctx().set_debug_on_hover(true);//设置debug
        // 先收集要删除的节点和新增的节点
        self.nodes.retain(|node_id, node| {
            let response = node.draw(ui, pan_zoom, &mut graph_state.node_state[node_id]);

            match response {
                NodeResponse::DeleteNode(id) => {
                    nodes_to_remove.push(id.clone()); // 收集要删除的节点
                    false // 标记为需要从 HashMap 中移除
                }
                NodeResponse::AddNode(id) => {
                    nodes_to_add.push(id.clone()); // 收集要新增的节点
                    true
                }
                _ => {
                    graph_response.nodes_response.insert_with_key(|mut k| {
                        k = node_id;
                        response
                    });
                    true
                }
            }
        });

        // 在 retain 操作完成后，处理收集到的删除和新增操作
        for id in nodes_to_remove {
            graph_state.node_state.remove(id); // 删除节点状态
            graph_state.graph_button_state.remove(id);// 删除节点状态
        }

        for id in nodes_to_add {
            let new_node = self.add_node_with_father_node(id.clone());
            graph_state
                .node_state
                .insert_with_key(|mut k| {
                k = new_node;
                NodeState::UnSelected
            }); // 插入新的节点状态
            graph_state.graph_button_state.insert_with_key(|mut k| {
                k = new_node;
                ButtonState::UnFold
            }); // 插入新的节点状态
        }
        // 判断并绘制节点的展开按钮
        for node in self.nodes.values(){
            if let Some(id)=node.father_id{
                if !nodes_have_children.contains(&id) {
                    nodes_have_children.push(id.clone());//如果id 不存在，才push 进去
                }
            }
        }
        for id in nodes_have_children{
            self.nodes[id].draw_button(ui, pan_zoom, &mut graph_state.graph_button_state[id]);
        }

        graph_response
    }
    fn add_node_with_father_node(&mut self, father_node_id:NodeId)->NodeId{
        let mut node_pos=self.nodes[father_node_id].node_pos+pos2(300.0,0.0).to_vec2();//默认往右平移300，后面需要算法计算位置

        let nodeid=self.nodes.insert_with_key(|node_id| {
            Node {
                node_id,
                transform: TSTransform::default(),
                node_pos,
                node_size: Vec2::new(100.0, 50.0),
                node_text: String::from("text"),
                node_color: Color32::from_rgb(150, 150, 250),
              //  node_state: NodeState::UnSelected,
                father_id: Some(father_node_id),
                button_pos:node_pos+ Vec2::new(100.0, 0.0) / 2.0 + Vec2::new(10.0, 0.0)
            }
        });
        nodeid
    }
    fn add_node_with_position(&mut self,pos2: Pos2)->NodeId {

         let nodeid=self.nodes.insert_with_key(|node_id| {
                Node {
                    node_id,
                    transform: TSTransform::default(),
                    node_pos: pos2,
                    node_size: Vec2::new(100.0, 50.0),
                    node_text: String::from("text"),
                    node_color: Color32::from_rgb(150, 150, 250),
                   // node_state: NodeState::UnSelected,
                    father_id: None,
                    button_pos:pos2+ Vec2::new(100.0, 0.0) / 2.0 + Vec2::new(10.0, 0.0)
                }
         });
        nodeid
    }
    fn select_node(& mut self, node_id: NodeId) {
        todo!()
    }
    fn edite_node(&mut self, node_id: NodeId) {
        todo!()
    }
    fn delete_node(&mut self, node_id: NodeId) {
        todo!()
    }
}
pub fn draw_bezier_line( stroke:Stroke,color:Color32,control_point:[Pos2; 4])->CubicBezierShape{
    let curve=CubicBezierShape::from_points_stroke(control_point,false,color,stroke);
    curve
}