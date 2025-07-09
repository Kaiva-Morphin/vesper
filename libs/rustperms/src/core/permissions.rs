use std::collections::HashMap;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

// todo: use bytes with limited alph instead
pub type PermissionPart = String;
pub type PermissionPath = SmallVec<[PermissionPart; 6]>;
pub type PermissionRule = (PermissionPath, bool);

pub trait PermPath {
    fn from_str(path: &str) -> Self;
    fn format(&self) -> String;
}

impl PermPath for PermissionPath {
    fn from_str(path: &str) -> PermissionPath {
        path.split('.').map(|s| s.to_string()).collect()
    }
    fn format(&self) -> String {
        self.join(".")
    }
}

pub trait ErrMap<T, E> {
    fn err_map(self) -> anyhow::Result<T>;
    fn err_msg(self, msg: &'static str) -> anyhow::Result<T>;
}

impl<T, E> ErrMap<T, E> for Result<T, E>
{
    fn err_map(self) -> anyhow::Result<T> {
        match self {
            Ok(v) => Ok(v),
            Err(_e) => Result::Err(anyhow::Error::msg("Error")),
        }
    }
    fn err_msg(self, msg: &'static str) -> anyhow::Result<T> {
        self.map_err(|_e| anyhow::Error::msg(msg))
    }
}

// TODO: CUSTOM SERIALIZER
#[derive(Serialize, Deserialize)] 
#[derive(Clone, Debug)]
#[derive(PartialEq, Eq)]
pub struct PermissionRuleNode {
    children: HashMap<PermissionPart, PermissionRuleNode>,
    enabled: Option<bool>
}

impl Default for PermissionRuleNode {
    fn default() -> Self {
        Self::new()
    }
}

impl PermissionRuleNode {
    pub fn new() -> Self {
        Self {
            children: HashMap::new(),
            enabled: None,
        }
    }

    pub fn remove(&mut self, path: &PermissionPath) {
        fn rec<'a>(node: &mut PermissionRuleNode, mut path: impl Iterator<Item = &'a PermissionPart> + Clone) -> bool {
            let Some(part) = path.next() else {
                return node.children.is_empty()
            };
            let Some(child) = node.children.get_mut(part) else {return false};
            if rec(child, path) {
                node.children.remove(part);
            }
            node.children.is_empty()
        }
        let path = path.iter();
        rec(self, path);
    }
    pub fn set(&mut self, path: PermissionPath, enabled: bool) {
        let mut current = self;
        for part in path {
            current = current.children.entry(part).or_default();
        }
        current.enabled = Some(enabled);
    }
    pub fn get(&self, path: &PermissionPath) -> Option<bool> {
        fn rec<'a>(
            node: &PermissionRuleNode,
            mut path: impl Iterator<Item = &'a PermissionPart> + Clone,
        ) -> Option<bool> {
            let Some(current) = path.next() else {
                return node.enabled;
            };

            // Exact match
            if let Some(child) = node.children.get(current) {
                if let Some(result) = rec(child, path.clone()) {
                    return Some(result);
                }
            }

            // ? matches exactly one part
            if let Some(child) = node.children.get("?") {
                if let Some(result) = rec(child, path.clone()) {
                    return Some(result);
                }
            }

            // * matches one or more parts
            if let Some(child) = node.children.get("*") {
                let mut tail = path.clone();
                let mut next = Some(current);
                while next.is_some() {
                    if let Some(result) = rec(child, tail.clone()) {
                        return Some(result);
                    }
                    next = tail.next();
                };
            }

            None
        }

        rec(self, path.iter())
    }

    pub fn get_records(&self) -> Vec<PermissionPath> {
        let mut records: Vec<PermissionPath> = Vec::new();
        if let Some(enabled) = self.enabled {
            let mut v = SmallVec::new();
            v.push(enabled.to_string());
            records.push(v);
        }
        for (key, child) in self.children.iter() {
            for record in child.get_records() {
                let mut new_record = SmallVec::with_capacity(record.len() + 1);
                new_record.push(key.clone());
                new_record.extend(record);
                records.push(new_record);
            }
        }
        records
    }

    pub fn merge(&mut self, other: Self) {
        if other.enabled.is_some() {
            self.enabled = other.enabled;
        }

        for (key, other_child) in other.children {
            self.children
                .entry(key)
                .or_default()
                .merge(other_child);
        }
    }

}

pub trait PermissionInterface {
    fn set_perm(&mut self, path: PermissionPath, enabled: bool);
    fn set_perms(&mut self, perms: Vec<PermissionRule>);
    fn remove_perm(&mut self, path: &PermissionPath);
    fn remove_perms(&mut self, perms: Vec<PermissionPath>);
    fn get_perm(&self, path: &PermissionPath) -> Option<bool>;
    fn get_perms(&self) -> &PermissionRuleNode;
    fn get_records(&self) -> Vec<PermissionPath> {self.get_perms().get_records()}
    fn merge(&mut self, other: Self);
}



#[test]
fn test_perm_path() {
    assert_eq!(PermissionPath::from_str("a.b.c"), ["a", "b", "c"].into());
    assert_eq!(PermissionPath::from_str("a.b.c.d"), ["a", "b", "c", "d"].into());
    assert_eq!(PermissionPath::from_str("a.b.c.d.e"), ["a", "b", "c", "d", "e"].into());
}
#[test]
fn test_perm_path_format() {
    assert_eq!(PermissionPath::from_str("a.b.c").format(), "a.b.c");
    assert_eq!(PermissionPath::from_str("a.b.c.d").format(), "a.b.c.d");
    assert_eq!(PermissionPath::from_str("a.b.c.d.e").format(), "a.b.c.d.e");
}

#[test]
fn test_set() {
    let mut tree = PermissionRuleNode::new();
    let p1 = PermissionPath::from_str("a.b.c");
    tree.set(p1.clone(), true);
    assert_eq!(tree.get(&p1), Some(true));
    tree.set(p1.clone(), false);
    assert_eq!(tree.get(&p1), Some(false));
}


#[test]
fn test_remove() {
    let mut tree = PermissionRuleNode::new();
    let p1 = PermissionPath::from_str("a.b.c");
    tree.set(p1.clone(), true);
    tree.remove(&p1);
    assert_eq!(tree.get(&p1), None);
}

#[test]
fn test_any_at_beginning() {
    let mut tree = PermissionRuleNode::new();
    tree.set(PermissionPath::from_str("?.b.c"), true);
    assert_eq!(tree.get(&PermissionPath::from_str("a.b.c")), Some(true));
    assert_eq!(tree.get(&PermissionPath::from_str("a.a.b.c")), None);
}

#[test]
fn test_any_in_middle() {
    let mut tree = PermissionRuleNode::new();
    tree.set(PermissionPath::from_str("a.?.c"), true);
    assert_eq!(tree.get(&PermissionPath::from_str("a.b.c")), Some(true));
    assert_eq!(tree.get(&PermissionPath::from_str("a.b.b.c")), None);
}

#[test]
fn test_any_at_end() {
    let mut tree = PermissionRuleNode::new();
    tree.set(PermissionPath::from_str("a.b.?"), true);
    assert_eq!(tree.get(&PermissionPath::from_str("a.b.c")), Some(true));
    assert_eq!(tree.get(&PermissionPath::from_str("a.b.c.d")), None);
}


#[test]
fn test_wildcard_root() {
    let mut tree = PermissionRuleNode::new();
    tree.set(PermissionPath::from_str("*"), true);
    assert_eq!(tree.get(&PermissionPath::from_str("x")), Some(true));
    assert_eq!(tree.get(&PermissionPath::from_str("x.y")), Some(true));
    assert_eq!(tree.get(&PermissionPath::from_str("x.y.z")), Some(true));
}

#[test]
fn test_wildcard_at_beginning() {
    let mut tree = PermissionRuleNode::new();
    tree.set(PermissionPath::from_str("*.b.c"), true);
    assert_eq!(tree.get(&PermissionPath::from_str("a.b.c")), Some(true));
    assert_eq!(tree.get(&PermissionPath::from_str("x.y.b.c")), Some(true));
    assert_eq!(tree.get(&PermissionPath::from_str("b.c")), None);
}

#[test]
fn test_wildcard_in_middle() {
    let mut tree = PermissionRuleNode::new();
    tree.set(PermissionPath::from_str("a.*.d"), true);
    assert_eq!(tree.get(&PermissionPath::from_str("a.b.c.d")), Some(true));
    assert_eq!(tree.get(&PermissionPath::from_str("a.b.d")), Some(true));
    assert_eq!(tree.get(&PermissionPath::from_str("a.d")), None);
}

#[test]
fn test_wildcard_at_end() {
    let mut tree = PermissionRuleNode::new();
    tree.set(PermissionPath::from_str("a.b.*"), true);
    assert_eq!(tree.get(&PermissionPath::from_str("a.b.c")), Some(true));
    assert_eq!(tree.get(&PermissionPath::from_str("a.b.c.d")), Some(true));
    assert_eq!(tree.get(&PermissionPath::from_str("a.b")), None);
}

#[test]
fn test_wildcards() {
    let mut tree = PermissionRuleNode::new();
    tree.set(PermissionPath::from_str("a.*.c.?.e"), true);
    assert_eq!(tree.get(&PermissionPath::from_str("a.x.y.c.z.e")), Some(true));
    assert_eq!(tree.get(&PermissionPath::from_str("a.x.c.z.e")), Some(true));
    assert_eq!(tree.get(&PermissionPath::from_str("a.c.z.e")), None);
    assert_eq!(tree.get(&PermissionPath::from_str("a.x.y.c.z.q.e")), None);
}

#[test]
fn test_merge_trees() {
    let mut base = PermissionRuleNode::new();
    let mut other = PermissionRuleNode::new();

    base.set(PermissionPath::from_str("a.b.c"), true);

    other.set(PermissionPath::from_str("a.b.c"), false);

    other.set(PermissionPath::from_str("a.x"), true);

    base.merge(other);

    assert_eq!(base.get(&PermissionPath::from_str("a.b.c")), Some(false));
    assert_eq!(base.get(&PermissionPath::from_str("a.x")), Some(true));
}

#[test]
fn test_merge_nested_wildcards() {
    let mut base = PermissionRuleNode::new();
    let mut other = PermissionRuleNode::new();

    base.set(PermissionPath::from_str("a.*.c"), true);
    other.set(PermissionPath::from_str("a.*.c"), false);

    base.merge(other);

    assert_eq!(base.get(&PermissionPath::from_str("a.b.c")), Some(false));
}
