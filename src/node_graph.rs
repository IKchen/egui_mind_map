use std::cmp::PartialEq;
use std::option::Option;
use std::{fmt, thread};
use eframe::emath::Vec2;
use eframe::epaint::{Pos2, pos2, Rounding};
use egui::{Area, Color32, Id, Shape, Stroke, Style, Ui};
use egui::emath::TSTransform;
use egui::epaint::CubicBezierShape;
use slotmap::SlotMap;
use crate::node::{ButtonResponse, ButtonState, Node, NodeResponse, NodeState};
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
    pub nodes_response:SlotMap<NodeId, NodeResponse>,
    pub buttons_response:SlotMap<NodeId, ButtonResponse>,
}
impl Default for GraphResponse{
    fn default() -> Self {
        Self{nodes_response:SlotMap::default(),buttons_response:SlotMap::default()}
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

impl PartialEq for NodeState {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (NodeState::Invisible, NodeState::Invisible) => true,
            (NodeState::Visible, NodeState::Visible) => true,
            (NodeState::Hover, NodeState::Hover) => true,
            (NodeState::Editing, NodeState::Editing) => true,
            (NodeState::Selected, NodeState::Selected) => true,
            (NodeState::UnSelected, NodeState::UnSelected) => true,
            _ => false,
        }
    }
}


impl NodeGraph{

    pub fn draw_curve_line(&mut self, ui: &mut Ui, pan_zoom: &PanZoom,graph_state: &mut GraphState) {

        for node in self.nodes.values() {
            if graph_state.node_state[node.node_id]!=NodeState::Invisible {
                let father_id = node.father_id;
                match father_id {
                    None => {} //没有节点时，不做任何事情
                    Some(father_id) => {
                        let father_node = self.nodes.get(father_id).expect("father id get wrong");
                        let control_points = [
                            node.node_pos - node.node_size * Vec2::new(0.5, 0.0),
                            pos2((node.node_pos.x + father_node.button_pos.x) / 2.0, node.button_pos.y),
                            pos2((node.node_pos.x + father_node.button_pos.x) / 2.0, father_node.button_pos.y),
                            father_node.button_pos + Vec2::new(10.0, 10.0) * Vec2::new(0.5, 0.0),
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
    //查询下一级子节点合集
    pub fn query_next_children_nodes(&mut self,father_node_id:NodeId) -> Vec<NodeId>{
        let mut nodes_children = Vec::new();
        for node in self.nodes.values() {
            if  let Some(node_father)=node.father_id{
                if node_father==father_node_id {
                    nodes_children.push(node.node_id.clone());
                }
            }
        }
        nodes_children
    }
    //查询所有子节点合集
    pub fn query_all_children_nodes(&mut self, father_node_id: NodeId) -> Vec<NodeId> {
        // 初始化结果向量，首先加入直接的子节点
        let mut nodes_all_children = self.query_next_children_nodes(father_node_id.clone());

        // 用一个栈结构模拟递归过程
        let mut stack = nodes_all_children.clone();  // 初始化栈，加入所有直接子节点

        // 迭代查找每个子节点的子节点
        while let Some(child_node_id) = stack.pop() {
           // println!("Processing node: {:?}", child_node_id);  // 添加打印以调试问题
            // 查询该子节点的所有子节点
            let mut child_descendants = self.query_next_children_nodes(child_node_id.clone());
            // 检查是否有环，避免重复处理同一节点
            child_descendants.retain(|id| !nodes_all_children.contains(id));

            // 将子节点的所有后代加入到总的结果集中
            nodes_all_children.extend(child_descendants.clone());

            // 将新的后代加入到栈中，继续处理这些后代的子节点
            stack.extend(child_descendants);

        }

        nodes_all_children
    }

}

impl NodeGraphs for NodeGraph{
    fn draw(&mut self, ui: &mut Ui, pan_zoom: &mut PanZoom, graph_state: &mut GraphState) -> GraphResponse {
        let mut graph_response = GraphResponse::default();
     
      
        ui.ctx().set_debug_on_hover(true);//设置debug
   
         // 绘制节点
         for (node_id, node) in &mut self.nodes {
            let should_draw = match node.father_id {
                None => true,
                Some(father_id) => graph_state.graph_button_state.get(father_id) == Some(&ButtonState::UnFold),
            };

            if should_draw {
                let response = node.draw(ui, pan_zoom, &mut graph_state.node_state[node_id]);
                graph_response.nodes_response.insert_with_key(|mut k| {
                    k = node_id;
                    response
                });
            }
        }

       
        //收集有子节点的节点ID
       let nodes_have_children: Vec<NodeId> = self.nodes.values()
       .filter_map(|node| node.father_id)
       .collect::<std::collections::HashSet<_>>()
       .into_iter()
       .collect();

        // 绘制展开按钮并收集响应
        for &id in &nodes_have_children {
        if let Some(node) = self.nodes.get_mut(id) {
        let button_response = node.draw_button(ui, pan_zoom, &mut graph_state.graph_button_state[id]);
        graph_response.buttons_response.insert_with_key(|mut k| {
            k = id;
            button_response
            });
            }
        }

        self.draw_curve_line(ui,pan_zoom,graph_state);
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
