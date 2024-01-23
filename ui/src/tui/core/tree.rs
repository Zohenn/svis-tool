use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};
use std::fmt::Debug;
use std::ops::Add;

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

#[derive(Hash, PartialEq, Eq, PartialOrd, Debug, Clone, Copy)]
enum TreeItemType {
    Node,
    Leaf,
}

#[derive(Hash, PartialEq, Eq, PartialOrd, Debug, Clone)]
struct TreeNodeChildKey {
    key: String,
    r#type: TreeItemType,
}

impl Ord for TreeNodeChildKey {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.r#type == other.r#type {
            return self.key.cmp(&other.key);
        } else if matches!(self.r#type, TreeItemType::Node) {
            return std::cmp::Ordering::Less;
        } else {
            return std::cmp::Ordering::Greater;
        }
    }
}

#[derive(Debug)]
pub struct TreeNode<D: Debug> {
    key: String,
    path: String,
    children: BTreeMap<TreeNodeChildKey, TreeItem<D>>,
}

pub struct Tree<D: Debug, A: Add<Output = A> + Copy> {
    pub items: TreeItem<D>,
    aggregated_data: HashMap<String, A>,
    aggregation_mapper: Option<Box<dyn Fn(&A) -> Vec<Span>>>,
}

#[derive(Clone, Copy)]
pub struct NoAggregation;

impl Add for NoAggregation {
    type Output = NoAggregation;

    fn add(self, rhs: Self) -> Self::Output {
        rhs
    }
}

impl<D: Debug> Tree<D, NoAggregation> {
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
                    let map_key = TreeNodeChildKey {
                        key: part,
                        r#type: TreeItemType::Node,
                    };

                    let new_node_entry = node.children.entry(map_key.clone()).or_insert_with(|| {
                        let path = if node.path.is_empty() {
                            map_key.key.clone()
                        } else {
                            node.path.clone() + "/" + &map_key.key
                        };
                        TreeItem::Node(TreeNode {
                            key: map_key.key.clone(),
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

            node.children.insert(
                TreeNodeChildKey {
                    key: leaf,
                    r#type: TreeItemType::Leaf,
                },
                TreeItem::Leaf(item),
            );
        }

        Tree {
            items: TreeItem::Node(root_node),
            aggregated_data: HashMap::new(),
            aggregation_mapper: None,
        }
    }

    pub fn with_aggregator<A: Add<Output = A> + Copy>(
        self,
        aggregator: impl Fn(&D) -> A + 'static,
        aggregation_mapper: impl Fn(&A) -> Vec<Span> + 'static,
    ) -> Tree<D, A> {
        let aggregated_data = aggregate(&self.items, aggregator);
        Tree {
            items: self.items,
            aggregated_data,
            aggregation_mapper: Some(Box::new(aggregation_mapper)),
        }
    }
}

impl<D: Debug, A: Add<Output = A> + Copy> Tree<D, A> {
    pub fn as_list_items(
        &self,
        state: &TreeState,
        data_mapper: impl Fn(&D) -> Vec<Span>,
    ) -> (Vec<String>, Vec<ListItem>) {
        let mut paths = vec![];
        let mut items = vec![];

        let mut queue: VecDeque<(u8, &TreeItem<D>)> = VecDeque::new();

        match &self.items {
            TreeItem::Node(node) => queue.extend(node.children.values().rev().map(|child| (0, child))),
            TreeItem::Leaf(_) => queue.push_back((0, &self.items)),
        }

        while let Some((depth, tree_item)) = queue.pop_back() {
            let padding = " ".repeat((depth as usize) * 2);
            match tree_item {
                TreeItem::Node(child_node) => {
                    let is_expanded = state.expanded.contains(&child_node.path);
                    let icon = if is_expanded { "▼ " } else { "► " };

                    paths.push(child_node.path.clone());

                    let mut line_contents = vec![padding.clone().into(), icon.into(), (&child_node.key).into()];

                    match (&self.aggregation_mapper, self.aggregated_data.get(&child_node.path)) {
                        (Some(aggregation_mapper), Some(aggregation)) => {
                            line_contents.push(" ".into());
                            line_contents.append(&mut aggregation_mapper(aggregation));
                        }
                        _ => {}
                    }

                    items.push(ListItem::new(Line::from(line_contents)));

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

fn aggregate<D: Debug, A: Add<Output = A> + Copy>(
    tree_item: &TreeItem<D>,
    aggregator: impl Fn(&D) -> A,
) -> HashMap<String, A> {
    let mut aggregated_data: HashMap<String, A> = HashMap::new();

    aggregate_inner(tree_item, &aggregator, &mut aggregated_data);

    aggregated_data
}

fn aggregate_inner<D: Debug, A: Add<Output = A> + Copy>(
    tree_item: &TreeItem<D>,
    aggregator: &impl Fn(&D) -> A,
    aggregated_data: &mut HashMap<String, A>,
) -> A {
    match tree_item {
        TreeItem::Node(node) => {
            let mut iter = node.children.values();
            let mut aggregation = aggregate_inner(iter.next().unwrap(), aggregator, aggregated_data);

            for child in iter {
                aggregation = aggregation + aggregate_inner(child, aggregator, aggregated_data);
            }

            aggregated_data.insert(node.path.clone(), aggregation);
            return aggregation;
        }
        TreeItem::Leaf(data) => {
            return aggregator(data);
        }
    }
}
