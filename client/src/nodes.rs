use rand::Rng;
use std::collections::HashSet;

pub struct Node<'a> {
    pub id: i32,

    pub hostname: &'a str,
    pub address: &'a str,
}

impl<'a> Node<'a> {
    pub fn new(ids: &mut HashSet<i32>, hostname: &'a str, address: &'a str) -> Self {
        let mut rng = rand::rng();
        let mut id: i32;

        loop {
            id = rng.random_range(1..=9999);
            if !ids.contains(&id) {
                ids.insert(id);
                break;
            }
        }

        Self {
            id,
            hostname,
            address,
        }
    }
}
