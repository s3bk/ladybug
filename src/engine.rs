use std::ops::{Not, Index, IndexMut};

use rand::prelude::SliceRandom;
use shakmaty::{Color, Outcome, Position};

use crate::board::Bughouse;

struct Node {
    side_that_moved: Color,
    position: Bughouse,
    wins: f32,
    simulations: i32,
    children: Vec<NodeId>,
}

#[derive(Copy, Clone)]
struct NodeId(usize);

struct Tree {
    nodes: Vec<Node>
}
impl Index<NodeId> for Tree {
    type Output = Node;
    fn index(&self, idx: NodeId) -> &Node {
        &self.nodes[idx.0]
    }
}
impl IndexMut<NodeId> for Tree {
    fn index_mut(&mut self, idx: NodeId) -> &mut Node {
        &mut self.nodes[idx.0]
    }
}

impl Tree {
    fn push_node(&mut self, node: Node) -> NodeId {
        let idx = self.nodes.len();
        self.nodes.push(node);
        NodeId(idx)
    }
    fn select_next(&self, node_id: NodeId) -> Option<NodeId> {
        let node = &self[node_id];
        let exploration_constant = 1.414; // sqrt(2) is theoretically ideal, but in practice this value is adjusted to maximize strength
        let uct = |child_id: NodeId| {
            let child = &self[child_id];
            if child.simulations == 0 {
                // Suggestions from around the internet say that the UCT score for unvisited nodes should be very high
                f32::MAX
            } else {
                child.wins / child.simulations as f32
                    + exploration_constant
                        * ((node.simulations as f32).ln() / child.simulations as f32).sqrt()
            }
        };
        node.children
            .iter()
            .fold(
                (None, -1f32),
                |(highest_uct_child, highest_uct): (Option<NodeId>, f32), &child_id| {
                    let uct_score = uct(child_id);
                    if uct_score > highest_uct {
                        (Some(child_id), uct_score)
                    } else {
                        (highest_uct_child, highest_uct)
                    }
                },
            )
            .0
    }

    // Selects an array of nodes from the root down to a leaf
    fn select_branch(&self, root: NodeId) -> Vec<NodeId> {
        let mut branch = vec![root];
        while let Some(next) = self.select_next(*branch.last().unwrap()) {
            branch.push(next);
        }
        branch
    }

    fn expand_tree(&mut self, node_id: NodeId) {
        let node = &mut self[node_id];
        let children: Vec<_> = node.position.legal_moves().iter().map(|legal_move| {
            Node {
                side_that_moved: node.side_that_moved.not(),
                position: node
                    .position
                    .clone()
                    .play(&legal_move)
                    .expect("Illegal move played from legal move list"),
                wins: 0f32,
                simulations: 0,
                children: vec![],
            }
        }).collect();
        let children_ids: Vec<_> = children.into_iter().map(|node| self.push_node(node)).collect();

        self[node_id].children.extend(children_ids);
    }

    fn simulate(position: Bughouse) -> Outcome {
        let mut simulation_board = position.clone();
        loop {
            if let Some(random_move) = simulation_board
                .legal_moves()
                .choose(&mut rand::thread_rng())
            {
                simulation_board = simulation_board
                    .play(random_move)
                    .expect("Illegal move played from legal move list");
            } else if let Some(outcome) = simulation_board.outcome() {
                break outcome;
            } else {
                panic!(
                    "No legal moves were found, but the game is not over (this should be impossible)"
                );
            }
        }
    }

    fn backpropagate(branch: Vec<&mut Node>, result: Outcome) {
        for node in branch {
            node.wins += match result {
                Outcome::Decisive { winner } => {
                    if winner == node.side_that_moved {
                        1f32
                    } else {
                        0f32
                    }
                }
                Outcome::Draw => 0.5f32,
            }
        }
    }

    fn execute_mcts(&mut self, root: NodeId) {
        let mut branch = self.select_branch(root);
        let leaf = branch.pop().expect("Branch should not be empty");
        self.expand_tree(leaf);
        //
        branch.push(leaf);
    }
}