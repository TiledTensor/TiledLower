use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::vec::Vec;

use crate::dataflow::{ThrillerEdge, ThrillerNode, ThrillerNodeInner};
use crate::task::Task;
use crate::{debug, AttachedEdge};
use crate::{next_id, MemoryLevel, ThrillerResult};

use super::loop_analysis::LoopGroup;

/// Thriller Dataflow Graph structure.
#[derive(Default)]
pub struct ThrillerGraph {
    #[allow(dead_code)]
    id: usize,
    nodes: Vec<Rc<RefCell<ThrillerNode>>>,
    edges: Vec<Rc<ThrillerEdge>>,
    #[allow(dead_code)]
    mem_level: MemoryLevel,
}

impl ThrillerGraph {
    /// Create a new empty ThrillerGraph.
    pub fn new(mem_level: MemoryLevel) -> Self {
        ThrillerGraph {
            id: next_id(),
            nodes: Vec::new(),
            edges: Vec::new(),
            mem_level,
        }
    }

    /// Add nodes into the graph.
    pub fn add_nodes(&mut self, nodes: Vec<Rc<RefCell<ThrillerNode>>>) {
        self.nodes.extend(nodes);
    }

    /// Add edges into the graph.
    pub fn add_edges(&mut self, edges: Vec<Rc<ThrillerEdge>>) {
        self.edges.extend(edges);
    }

    /// Connect the nodes in the graph.
    pub fn connect(&mut self) {
        for edge in &self.edges {
            let src = edge.get_src();
            let dst = edge.get_dst();

            let mut src_ref = src.borrow_mut();
            let mut dst_ref = dst.borrow_mut();

            src_ref.add_out_edge(edge.clone());
            dst_ref.add_in_edge(edge.clone());

            src_ref.add_next(dst.clone());
            dst_ref.add_prev(src.clone());

            dst_ref.inc_in_degrees();
        }
    }

    /// Topological sort the nodes in the graph.
    pub fn topo_sort(&self) -> Vec<Rc<RefCell<ThrillerNode>>> {
        let mut sorted_nodes = Vec::new();
        // (id, (in_degrees, node))
        let mut in_degrees: HashMap<usize, (usize, &Rc<RefCell<ThrillerNode>>)> = HashMap::new();

        for node in &self.nodes {
            let ref_node = node.borrow();
            in_degrees.insert(ref_node.get_id(), (ref_node.get_in_degrees(), node));
            debug!(
                "{} have {} in_degrees.",
                ref_node.get_id(),
                ref_node.get_in_degrees()
            );
        }

        while !in_degrees.is_empty() {
            let node_ids = in_degrees.keys().cloned().collect::<Vec<_>>();
            for node_id in node_ids {
                let (in_degree, node) = in_degrees[&node_id];
                if in_degree == 0 {
                    sorted_nodes.push(node.clone());

                    for next in node.borrow_mut().get_nexts() {
                        let next_id = next.borrow().get_id();
                        let (in_degree, _) = in_degrees.get_mut(&next_id).unwrap();
                        *in_degree -= 1;
                    }

                    in_degrees.remove(&node_id);
                }
            }
        }

        sorted_nodes
    }

    /// Reduce the block outputs in the graph.
    pub fn reduce_block_outputs(&self) -> Option<Vec<Rc<AttachedEdge>>> {
        let sorted_nodes = self.topo_sort();

        for node in sorted_nodes {
            if let ThrillerNodeInner::Block(block) = node.borrow().get_inner() {
                let outputs = block.reduce();
                let mut reduced_outputs = Vec::new();
                if let Some(outputs) = outputs {
                    for output in outputs {
                        reduced_outputs.push(output.clone());
                    }
                    return Some(reduced_outputs);
                }
                return None;
            }
        }

        None
    }
}

impl ThrillerGraph {
    /// Try to find the disconnected subgraph in the graph.
    #[allow(dead_code)]
    pub(crate) fn try_find_disconnected_subgraph(&self) -> Option<Vec<ThrillerGraph>> {
        todo!("find_disconnected_subgraph not implemented yet");
    }

    /// Try to split graph based on various loop nests.
    #[allow(dead_code)]
    pub(crate) fn try_split(&mut self, groups: &[LoopGroup]) -> Option<Vec<ThrillerGraph>> {
        if groups.len() == 1 {
            return None;
        }

        todo!("try_split not implemented yet");
    }
}

impl Task for ThrillerGraph {
    fn emit(&self) -> ThrillerResult<String> {
        let mut code = String::new();
        let sorted_nodes = self.topo_sort();

        for node in sorted_nodes {
            match node.borrow().get_inner() {
                ThrillerNodeInner::Op(op) => {
                    code += op.emit()?.as_str();
                }
                ThrillerNodeInner::Block(block) => {
                    code += block.emit()?.as_str();
                }
                _ => {}
            }
        }

        Ok(code)
    }

    fn get_name(&self) -> String {
        todo!()
    }
}
