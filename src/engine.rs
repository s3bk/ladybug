use std::ops::Not;

use rand::prelude::SliceRandom;
use shakmaty::{Color, Outcome, Position};

use crate::board::Bughouse;

struct Node {
    side_that_moved: Color,
    position: Bughouse,
    wins: f32,
    simulations: i32,
    children: Vec<Node>,
}

fn select_next(node: &Node) -> Option<&Node> {
    let exploration_constant = 1.414; // sqrt(2) is theoretically ideal, but in practice this value is adjusted to maximize strength
    let uct = |child: &Node| {
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
            |(highest_uct_child, highest_uct): (Option<&Node>, f32), child| {
                let uct_score = uct(child);
                if uct_score > highest_uct {
                    (Some(child), uct_score)
                } else {
                    (highest_uct_child, highest_uct)
                }
            },
        )
        .0
}

// Selects an array of nodes from the root down to a leaf
fn select_branch(root: &Node) -> Vec<&Node> {
    let mut branch: Vec<&Node> = vec![root];
    while let Some(next) = select_next(branch.last().unwrap()) {
        branch.push(next);
    }
    branch
}

fn expand_tree(node: &mut Node) {
    for legal_move in node.position.legal_moves() {
        node.children.push(Node {
            side_that_moved: node.side_that_moved.not(),
            position: node
                .position
                .clone()
                .play(&legal_move)
                .expect("Illegal move played from legal move list"),
            wins: 0f32,
            simulations: 0,
            children: vec![],
        });
    }
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

fn execute_mcts(mut root: Node) {
    let mut branch = select_branch(&mut root);
    let leaf = branch.pop().expect("Branch should not be empty");
    expand_tree(&mut leaf);
    //
    branch.push(leaf);
}
