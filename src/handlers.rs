use crate::node_graph::{NodeGraph, GraphState, GraphResponse,NodeId};
use crate::node::{NodeResponse, ButtonResponse,NodeState,ButtonState};
use slotmap::SlotMap;
use crate::traits::*;
pub fn handle_graph_response(
    node_graph: &mut NodeGraph,
    graph_state: &mut GraphState,
    graph_response: GraphResponse
) -> Result<(), String> {
    handle_node_responses(node_graph, graph_state, &graph_response.nodes_response)?;
    handle_button_responses(node_graph, graph_state, &graph_response.buttons_response)?;
    Ok(())
}

fn handle_node_responses(
    node_graph: &mut NodeGraph,
    graph_state: &mut GraphState,
    nodes_response: &SlotMap<NodeId, NodeResponse>
) -> Result<(), String> {
    let mut nodes_to_remove = Vec::new();//要删除的节点
    let mut nodes_to_add = Vec::new();//需要新增的节点
    for (nodeid,response) in nodes_response{
        match response {
            NodeResponse::EditNode(id) => {
                graph_state.node_state[*id]=NodeState::Editing;
            }
            NodeResponse::AddNode(id)=> {
                nodes_to_add.push(*id); // 收集要新增的节点
            }
            NodeResponse::Selected (id)=> {
                graph_state.node_state[*id]=NodeState::Selected;
            }
            NodeResponse::UnSelected(id) => {
                graph_state.node_state[*id]=NodeState::UnSelected;
            }
            NodeResponse::InvisibleNode(id)=>{
                graph_state.node_state[*id]=NodeState::Invisible;
            }
            NodeResponse::VisibleNode(id)=>{
                graph_state.node_state[*id]=NodeState::Visible;
            }
            NodeResponse::DeleteNode(id)=>{
                nodes_to_remove.push(*id); // 收集要删除的节点
            }
            _ => {}
        }
    }
    //新增节点
    for id in nodes_to_add {
        let new_node = node_graph.add_node_with_father_node(id.clone());
        graph_state
            .node_state
            .insert_with_key(|mut k| {
            k = new_node;
            NodeState::UnSelected
        }); // 插入新的节点状态
        graph_state.graph_button_state.insert_with_key(|mut k| {
            k = new_node;
            ButtonState::UnFold
        }); // 插入新的节点 按钮状态
    }

    //删除节点
    for id in nodes_to_remove {
        graph_state.node_state.remove(id); // 删除节点状态
        graph_state.graph_button_state.remove(id);// 删除节点按钮状态
    }
    Ok(())
}

fn handle_button_responses(
    node_graph: &mut NodeGraph,
    graph_state: &mut GraphState,
    buttons_response: &SlotMap<NodeId, ButtonResponse>
) -> Result<(), String> {
    for (node_id, response) in buttons_response {
        //根据 button 返回 button response的父级id 去查询 子级id的列表,并把所有子级 response 设为隐藏
        
        match response {
            ButtonResponse::FoldNode(father_id) => {
                println!("the button response is {:?}",father_id);
                graph_state.graph_button_state[*father_id]=ButtonState::Fold;
                
                let children = node_graph.query_all_children_nodes(*father_id);
                for child_id in children {
                    graph_state.node_state[child_id] = NodeState::Invisible;
                }
            },
            ButtonResponse::UnfoldNode(father_id) => {
                println!("the button response is {:?}",father_id);
                graph_state.graph_button_state[*father_id]=ButtonState::UnFold;
                let children = node_graph.query_all_children_nodes(*father_id);
                for child_id in children {
                    graph_state.node_state[child_id] = NodeState::Visible;
                }
            },
            ButtonResponse::None => {},
        }
    }
    Ok(())
}
