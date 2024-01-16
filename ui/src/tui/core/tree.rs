use std::collections::{BTreeMap, HashSet, VecDeque};
use std::fmt::Debug;

use ratatui::prelude::*;
use ratatui::widgets::ListItem;

pub struct TreeState {
    pub expanded: HashSet<String>,
}

#[derive(Debug)]
pub enum TreeItem<D: Debug> {
    Node(TreeNode<D>),
    Leaf(D),
}

#[derive(Debug)]
pub struct TreeNode<D: Debug> {
    key: String,
    path: String,
    children: BTreeMap<String, TreeItem<D>>,
}

pub struct Tree<D: Debug> {
    pub items: TreeItem<D>,
}

impl<D: Debug> Tree<D> {
    pub fn from(items: Vec<D>, get_path: impl Fn(&D) -> &str) -> Self {
        let mut root_node: TreeNode<D> = TreeNode {
            key: String::new(),
            path: String::new(),
            children: BTreeMap::new(),
        };

        for item in items {
            let mut path_parts = get_path(&item)
                .split('/')
                .map(|part| part.to_owned())
                .collect::<Vec<String>>();

            let leaf = path_parts.pop().unwrap();
            let mut node = &mut root_node;

            if path_parts.len() > 0 {
                for part in path_parts.into_iter() {
                    let new_node_entry = node.children.entry(part.clone()).or_insert_with(|| {
                        let path = if node.path.is_empty() {
                            part.clone()
                        } else {
                            node.path.clone() + "/" + &part
                        };
                        TreeItem::Node(TreeNode {
                            key: part.clone(),
                            path,
                            children: BTreeMap::new(),
                        })
                    });
                    node = match new_node_entry {
                        TreeItem::Node(node) => node,
                        _ => unreachable!(),
                    };
                }
            }

            node.children.insert(leaf, TreeItem::Leaf(item));
        }

        Tree {
            items: TreeItem::Node(root_node),
        }
    }

    pub fn as_list_items(
        &self,
        state: &TreeState,
        data_mapper: impl Fn(&D) -> Vec<Span>,
    ) -> (Vec<String>, Vec<ListItem>) {
        let mut paths = vec![];
        let mut items = vec![];

        let mut queue: VecDeque<(u8, &TreeItem<D>)> = VecDeque::new();

        match &self.items {
            TreeItem::Node(node) => {
                for child in node.children.values() {
                    queue.push_back((0, child));
                }
            }
            TreeItem::Leaf(_) => queue.push_back((0, &self.items)),
        }

        while let Some((depth, tree_item)) = queue.pop_back() {
            let padding = " ".repeat((depth as usize) * 2);
            match tree_item {
                TreeItem::Node(child_node) => {
                    let is_expanded = state.expanded.contains(&child_node.path);
                    let icon = if is_expanded { "▼ " } else { "► " };

                    paths.push(child_node.path.clone());
                    items.push(ListItem::new(Line::from(vec![
                        padding.clone().into(),
                        icon.into(),
                        (&child_node.key).into(),
                    ])));

                    if is_expanded {
                        for child in child_node.children.values().rev() {
                            queue.push_back((depth + 1, child));
                        }
                    }
                }
                TreeItem::Leaf(data) => {
                    paths.push(String::new());
                    let mut line_contents = vec![padding.clone().into(), "  ".into()];
                    line_contents.append(&mut data_mapper(data));
                    items.push(ListItem::new(Line::from(line_contents)));
                }
            }
        }

        (paths, items)
    }
}
