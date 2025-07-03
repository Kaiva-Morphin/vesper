use crate::prelude::{Group, PermissionInterface, PermissionPath};


pub trait AddBind<T> {
    fn add_bind(&mut self, with: T);
}

pub trait RemoveBind<T> {
    fn remove_bind(&mut self, with: T);
}

pub trait Set<T> {
    fn set(&mut self, to_set: T);
}

impl<T: PermissionInterface> Set<(PermissionPath, bool)> for T {
    fn set(&mut self, (path, enabled): (PermissionPath, bool)) {
        self.set_perm(path, enabled);
    }
}


impl Set<usize> for Group {
    fn set(&mut self, to_set: usize) {
        self.weight = to_set;
    }
}

pub trait Remove<T> {
    fn remove(&mut self, to_remove: T);
}

impl<T: PermissionInterface> Remove<&PermissionPath> for T {
    fn remove(&mut self, path: &PermissionPath) {
        self.remove_perm(path);
    }
}