use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq)]
pub enum CollectionTreeData<T> {
    Collection(Vec<T>),
    Tree(Vec<CollectionTreeNode<T>>),
}

impl<T> CollectionTreeData<T> {
    pub const fn is_tree(&self) -> bool {
        matches!(self, Self::Tree(_))
    }

    pub fn is_empty(&self) -> bool {
        match self {
            Self::Collection(items) => items.is_empty(),
            Self::Tree(nodes) => nodes.is_empty(),
        }
    }
}

impl<T: Clone> CollectionTreeData<T> {
    pub fn from_parented_collection<F, P>(
        items: Vec<T>,
        key_of: F,
        parent_key_of: P,
    ) -> Result<Self, String>
    where
        F: Fn(&T) -> String,
        P: Fn(&T) -> Option<String>,
    {
        let mut items_by_key = BTreeMap::new();
        let mut parent_by_key = BTreeMap::new();
        let mut ordered_keys = Vec::new();
        for item in items {
            let key = key_of(&item);
            validate_key(&key, &items_by_key)?;
            let parent_key = parent_key_of(&item).filter(|parent| !parent.trim().is_empty());
            if parent_key.as_deref() == Some(key.as_str()) {
                return Err(format!("项目不能以自身作为父项：{key}"));
            }
            ordered_keys.push(key.clone());
            parent_by_key.insert(key.clone(), parent_key);
            items_by_key.insert(key, item);
        }

        let mut children_by_key = BTreeMap::<String, Vec<String>>::new();
        let mut root_keys = Vec::new();
        for key in &ordered_keys {
            match parent_by_key.get(key).and_then(Clone::clone) {
                Some(parent_key) if items_by_key.contains_key(&parent_key) => {
                    children_by_key
                        .entry(parent_key)
                        .or_default()
                        .push(key.clone());
                }
                Some(_) | None => root_keys.push(key.clone()),
            }
        }

        let mut visiting = BTreeSet::new();
        let mut built = BTreeSet::new();
        let mut roots = Vec::new();
        for key in root_keys {
            roots.push(build_tree_node(
                &key,
                &items_by_key,
                &children_by_key,
                &mut visiting,
                &mut built,
            )?);
        }
        for key in ordered_keys {
            if !built.contains(&key) {
                build_tree_node(
                    &key,
                    &items_by_key,
                    &children_by_key,
                    &mut visiting,
                    &mut built,
                )?;
            }
        }
        Ok(Self::Tree(roots))
    }
}

fn validate_key<T>(key: &str, items_by_key: &BTreeMap<String, T>) -> Result<(), String> {
    if key.trim().is_empty() {
        return Err("项目键不能为空".to_owned());
    }
    if items_by_key.contains_key(key) {
        return Err(format!("项目键重复：{key}"));
    }
    Ok(())
}

fn build_tree_node<T: Clone>(
    key: &str,
    items_by_key: &BTreeMap<String, T>,
    children_by_key: &BTreeMap<String, Vec<String>>,
    visiting: &mut BTreeSet<String>,
    built: &mut BTreeSet<String>,
) -> Result<CollectionTreeNode<T>, String> {
    if !visiting.insert(key.to_owned()) {
        return Err(format!("检测到父子环：{key}"));
    }
    let item = items_by_key
        .get(key)
        .cloned()
        .ok_or_else(|| format!("找不到项目：{key}"))?;
    let mut children = Vec::new();
    if let Some(child_keys) = children_by_key.get(key) {
        for child_key in child_keys {
            children.push(build_tree_node(
                child_key,
                items_by_key,
                children_by_key,
                visiting,
                built,
            )?);
        }
    }
    visiting.remove(key);
    built.insert(key.to_owned());
    Ok(CollectionTreeNode { item, children })
}

#[derive(Clone, Debug, PartialEq)]
pub struct CollectionTreeNode<T> {
    pub item: T,
    pub children: Vec<Self>,
}

impl<T> CollectionTreeNode<T> {
    pub const fn leaf(item: T) -> Self {
        Self {
            item,
            children: Vec::new(),
        }
    }

    pub const fn branch(item: T, children: Vec<Self>) -> Self {
        Self { item, children }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ResolvedCollectionTreeItem<T> {
    pub item: T,
    pub key: String,
    pub depth: usize,
    pub has_children: bool,
    pub expanded: bool,
}

pub(crate) fn resolve_visible_items<T, F>(
    data: &CollectionTreeData<T>,
    collapsed_keys: &BTreeSet<String>,
    key_of: F,
) -> Result<Vec<ResolvedCollectionTreeItem<T>>, String>
where
    T: Clone,
    F: Fn(&T) -> String,
{
    let mut keys = BTreeSet::new();
    let mut rows = Vec::new();
    match data {
        CollectionTreeData::Collection(items) => {
            for item in items {
                let key = register_key(item, &key_of, &mut keys)?;
                rows.push(ResolvedCollectionTreeItem {
                    item: item.clone(),
                    key,
                    depth: 0,
                    has_children: false,
                    expanded: false,
                });
            }
        }
        CollectionTreeData::Tree(nodes) => {
            resolve_tree_nodes(nodes, 0, collapsed_keys, &key_of, &mut keys, &mut rows)?;
        }
    }
    Ok(rows)
}

fn resolve_tree_nodes<T, F>(
    nodes: &[CollectionTreeNode<T>],
    depth: usize,
    collapsed_keys: &BTreeSet<String>,
    key_of: &F,
    keys: &mut BTreeSet<String>,
    rows: &mut Vec<ResolvedCollectionTreeItem<T>>,
) -> Result<(), String>
where
    T: Clone,
    F: Fn(&T) -> String,
{
    for node in nodes {
        let key = register_key(&node.item, key_of, keys)?;
        let has_children = !node.children.is_empty();
        let expanded = has_children && !collapsed_keys.contains(&key);
        rows.push(ResolvedCollectionTreeItem {
            item: node.item.clone(),
            key,
            depth,
            has_children,
            expanded,
        });
        if expanded {
            resolve_tree_nodes(
                &node.children,
                depth + 1,
                collapsed_keys,
                key_of,
                keys,
                rows,
            )?;
        } else {
            register_hidden_keys(&node.children, key_of, keys)?;
        }
    }
    Ok(())
}

fn register_hidden_keys<T, F>(
    nodes: &[CollectionTreeNode<T>],
    key_of: &F,
    keys: &mut BTreeSet<String>,
) -> Result<(), String>
where
    F: Fn(&T) -> String,
{
    for node in nodes {
        register_key(&node.item, key_of, keys)?;
        register_hidden_keys(&node.children, key_of, keys)?;
    }
    Ok(())
}

fn register_key<T, F>(item: &T, key_of: &F, keys: &mut BTreeSet<String>) -> Result<String, String>
where
    F: Fn(&T) -> String,
{
    let key = key_of(item);
    if key.trim().is_empty() {
        return Err("项目键不能为空".to_owned());
    }
    if !keys.insert(key.clone()) {
        return Err(format!("项目键重复：{key}"));
    }
    Ok(key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, Eq, PartialEq)]
    struct Item(&'static str);

    #[test]
    fn collection_resolves_to_flat_items() -> Result<(), String> {
        let data = CollectionTreeData::Collection(vec![Item("model"), Item("page")]);
        let rows = resolve_visible_items(&data, &BTreeSet::new(), |item| item.0.to_owned())?;

        assert_eq!(rows.len(), 2);
        assert!(rows.iter().all(|row| row.depth == 0));
        assert!(rows.iter().all(|row| !row.has_children));
        Ok(())
    }

    #[test]
    fn tree_hides_descendants_when_branch_is_collapsed() -> Result<(), String> {
        let data = CollectionTreeData::Tree(vec![CollectionTreeNode::branch(
            Item("root"),
            vec![CollectionTreeNode::branch(
                Item("child"),
                vec![CollectionTreeNode::leaf(Item("leaf"))],
            )],
        )]);
        let collapsed = BTreeSet::from(["child".to_owned()]);
        let rows = resolve_visible_items(&data, &collapsed, |item| item.0.to_owned())?;

        assert_eq!(
            rows.iter().map(|row| row.key.as_str()).collect::<Vec<_>>(),
            vec!["root", "child"]
        );
        assert_eq!(rows[1].depth, 1);
        assert!(!rows[1].expanded);
        Ok(())
    }

    #[test]
    fn duplicate_keys_are_rejected_even_under_collapsed_branch() {
        let data = CollectionTreeData::Tree(vec![CollectionTreeNode::branch(
            Item("root"),
            vec![CollectionTreeNode::leaf(Item("root"))],
        )]);
        let collapsed = BTreeSet::from(["root".to_owned()]);

        let result = resolve_visible_items(&data, &collapsed, |item| item.0.to_owned());

        assert_eq!(result, Err("项目键重复：root".to_owned()));
    }

    #[derive(Clone, Debug, Eq, PartialEq)]
    struct ParentedItem {
        key: &'static str,
        parent: Option<&'static str>,
    }

    #[test]
    fn parented_collection_builds_tree_from_unordered_items() -> Result<(), String> {
        let data = CollectionTreeData::from_parented_collection(
            vec![
                ParentedItem {
                    key: "child",
                    parent: Some("root"),
                },
                ParentedItem {
                    key: "root",
                    parent: None,
                },
            ],
            |item| item.key.to_owned(),
            |item| item.parent.map(str::to_owned),
        )?;

        let CollectionTreeData::Tree(roots) = data else {
            return Err("集合未转换为树".to_owned());
        };
        assert_eq!(roots.len(), 1);
        assert_eq!(roots[0].item.key, "root");
        assert_eq!(roots[0].children[0].item.key, "child");
        Ok(())
    }

    #[test]
    fn parented_collection_rejects_cycles() {
        let result = CollectionTreeData::from_parented_collection(
            vec![
                ParentedItem {
                    key: "one",
                    parent: Some("two"),
                },
                ParentedItem {
                    key: "two",
                    parent: Some("one"),
                },
            ],
            |item| item.key.to_owned(),
            |item| item.parent.map(str::to_owned),
        );

        assert_eq!(result, Err("检测到父子环：one".to_owned()));
    }
}
