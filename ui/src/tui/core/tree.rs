use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};
use std::fmt::Debug;
use std::ops::Add;

use compact_str::CompactString;
use ratatui::prelude::*;
use ratatui::widgets::{ListItem, ListState};

use super::ListOperations;

#[derive(Default)]
pub struct TreeState {
    pub expanded: HashSet<String>,
    pub list_state: ListState,
    rendered: bool,
    initial_expansion_depth: u8,
    initial_highlight: Option<String>,
    paths: Vec<String>,
}

impl TreeState {
    pub fn initial_expansion_depth(mut self, depth: u8) -> Self {
        self.initial_expansion_depth = depth;
        self
    }

    pub fn initial_highlight(&mut self, path: &str) {
        self.initial_highlight = Some(path.to_owned());
    }

    #[allow(dead_code)]
    pub fn with_expanded(expanded: HashSet<String>) -> Self {
        Self {
            expanded,
            list_state: ListState::default(),
            rendered: false,
            initial_expansion_depth: 0,
            initial_highlight: None,
            paths: vec![],
        }
    }

    pub fn ensure_leaf_is_visible(&mut self, leaf: &str) {
        let parts = leaf.split('/').collect::<Vec<_>>();

        for index in 0..parts.len() {
            let current_path_parts = parts.get(0..=index).unwrap();
            self.expanded.insert(current_path_parts.join("/"));
        }
    }

    pub fn toggle_selected(&mut self) {
        let path = &self.paths[self.selected().unwrap_or(0)];

        if !path.is_empty() {
            if self.expanded.contains(path) {
                self.expanded.remove(path);
            } else {
                self.expanded.insert(path.clone());
            }
        }
    }
}

impl ListOperations for TreeState {
    fn len(&self) -> usize {
        self.paths.len()
    }

    fn selected(&self) -> Option<usize> {
        self.list_state.selected()
    }

    fn select_inner(&mut self, index: Option<usize>) {
        self.list_state.select(index)
    }
}

#[derive(Debug)]
pub enum TreeItem<D: Debug> {
    Node(TreeNode<D>),
    Leaf(TreeLeaf<D>),
}

#[derive(Hash, PartialEq, Eq, PartialOrd, Debug, Clone, Copy)]
enum TreeItemType {
    Node,
    Leaf,
}

#[derive(Hash, PartialEq, Eq, Debug, Clone)]
struct TreeNodeChildKey {
    key: CompactString,
    r#type: TreeItemType,
}

impl PartialOrd for TreeNodeChildKey {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TreeNodeChildKey {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.r#type == other.r#type {
            self.key.cmp(&other.key)
        } else if matches!(self.r#type, TreeItemType::Node) {
            std::cmp::Ordering::Less
        } else {
            std::cmp::Ordering::Greater
        }
    }
}

#[derive(Debug)]
pub struct TreeLocation {
    key: CompactString,
    path: String,
}

impl TreeLocation {
    pub fn new<K>(key: K, path: String) -> Self
    where
        K: Into<CompactString>,
    {
        Self { key: key.into(), path }
    }
}

#[derive(Debug)]
pub struct TreeNode<D: Debug> {
    location: TreeLocation,
    children: BTreeMap<TreeNodeChildKey, TreeItem<D>>,
}

#[derive(Debug)]
pub struct TreeLeaf<D: Debug> {
    location: TreeLocation,
    data: D,
}

pub struct Tree<D: Debug, A: Add<Output = A> + Copy> {
    pub items: TreeItem<D>,
    aggregated_data: HashMap<String, A>,
    #[allow(clippy::type_complexity)]
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
    pub fn from(items: Vec<D>, get_path: impl Fn(&D) -> String) -> Self {
        let mut root_node: TreeNode<D> = TreeNode {
            location: TreeLocation::new("", String::new()),
            children: BTreeMap::new(),
        };

        for item in items {
            let path = get_path(&item);
            let mut path_parts = path.split('/').map(|part| part.into()).collect::<Vec<CompactString>>();

            let leaf = path_parts.pop().unwrap();
            let mut node = &mut root_node;

            if !path_parts.is_empty() {
                for part in path_parts.into_iter() {
                    let map_key = TreeNodeChildKey {
                        key: part,
                        r#type: TreeItemType::Node,
                    };

                    let new_node_entry = node.children.entry(map_key.clone()).or_insert_with(|| {
                        let path = if node.location.path.is_empty() {
                            map_key.key.clone().to_string()
                        } else {
                            node.location.path.clone() + "/" + &map_key.key
                        };
                        TreeItem::Node(TreeNode {
                            location: TreeLocation::new(map_key.key.clone(), path),
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
                    key: leaf.clone(),
                    r#type: TreeItemType::Leaf,
                },
                TreeItem::Leaf(TreeLeaf {
                    location: TreeLocation::new(leaf, path),
                    data: item,
                }),
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
        leaf_aggregations: &[A],
        aggregator: impl Fn(&[A], &D) -> A,
        aggregation_mapper: impl Fn(&A) -> Vec<Span> + 'static,
    ) -> Tree<D, A> {
        let aggregated_data = aggregate(&self.items, leaf_aggregations, aggregator);
        Tree {
            items: self.items,
            aggregated_data,
            aggregation_mapper: Some(Box::new(aggregation_mapper)),
        }
    }
}

impl<D: Debug, A: Add<Output = A> + Copy> Tree<D, A> {
    pub fn as_list_items<'tree>(
        &'tree self,
        state: &mut TreeState,
        data_mapper: impl Fn(&D) -> Vec<Span<'tree>>,
    ) -> Vec<ListItem> {
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
                    if !state.rendered && depth < state.initial_expansion_depth {
                        state.expanded.insert(child_node.location.path.clone());
                    }

                    let is_expanded = state.expanded.contains(&child_node.location.path);
                    let icon = if is_expanded { "▼ " } else { "► " };

                    paths.push(child_node.location.path.clone());

                    let mut line_contents =
                        vec![padding.clone().into(), icon.into(), (&child_node.location.key).into()];

                    if let (Some(aggregation_mapper), Some(aggregation)) = (
                        &self.aggregation_mapper,
                        self.aggregated_data.get(&child_node.location.path),
                    ) {
                        line_contents.push(" ".into());
                        line_contents.append(&mut aggregation_mapper(aggregation));
                    }

                    items.push(ListItem::new(Line::from(line_contents)));

                    if is_expanded {
                        for child in child_node.children.values().rev() {
                            queue.push_back((depth + 1, child));
                        }
                    }
                }
                TreeItem::Leaf(leaf) => {
                    paths.push(leaf.location.path.clone());
                    let mut line_contents = vec![padding.clone().into(), "  ".into()];
                    line_contents.append(&mut data_mapper(&leaf.data));
                    items.push(ListItem::new(Line::from(line_contents)));
                }
            }

            if let Some(path) = &state.initial_highlight {
                let Some(last_path) = paths.last() else {
                    continue;
                };

                if last_path == path {
                    state.list_state.select(Some(paths.len() - 1));
                    state.initial_highlight = None;
                }
            };
        }

        state.rendered = true;
        state.paths = paths;

        items
    }
}

fn aggregate<D: Debug, A: Add<Output = A> + Copy>(
    tree_item: &TreeItem<D>,
    leaf_aggregations: &[A],
    aggregator: impl Fn(&[A], &D) -> A,
) -> HashMap<String, A> {
    let mut aggregated_data: HashMap<String, A> = HashMap::new();

    aggregate_inner(tree_item, leaf_aggregations, &aggregator, &mut aggregated_data);

    aggregated_data
}

fn aggregate_inner<D: Debug, A: Add<Output = A> + Copy>(
    tree_item: &TreeItem<D>,
    leaf_aggregations: &[A],
    aggregator: &impl Fn(&[A], &D) -> A,
    aggregated_data: &mut HashMap<String, A>,
) -> A {
    match tree_item {
        TreeItem::Node(node) => {
            let mut iter = node.children.values();
            let mut aggregation = aggregate_inner(iter.next().unwrap(), leaf_aggregations, aggregator, aggregated_data);

            for child in iter {
                aggregation = aggregation + aggregate_inner(child, leaf_aggregations, aggregator, aggregated_data);
            }

            aggregated_data.insert(node.location.path.clone(), aggregation);
            aggregation
        }
        TreeItem::Leaf(leaf) => aggregator(leaf_aggregations, &leaf.data),
    }
}
