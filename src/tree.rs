use crate::replace;
use git::{Commit, Oid, Repository};
use hashbrown::{HashMap, HashSet};

pub type Children<'a> = HashMap<Oid, Vec<Commit<'a>>>;

pub fn collect_children<'a>(repo: &'a Repository, mut end: Commit<'a>) -> Children<'a> {
    let mut children = HashMap::<Oid, HashSet<Oid>>::new();
    let mut equals = HashMap::<Oid, Oid>::new();

    if let Some(old_id) = replace(repo, &mut end) {
        equals.insert(old_id, end.id());
    }
    let mut stack = vec![(end, 0)];
    while let Some((commit, i)) = stack.last_mut() {
        if let Ok(mut parent) = commit.parent(*i) {
            *i += 1;
            let mut done = true;
            if let Some(old_id) = replace(repo, &mut parent) {
                equals.insert(old_id, parent.id());
            }
            children
                .entry(parent.id())
                .or_insert_with(|| {
                    done = false;
                    HashSet::new()
                })
                .insert(commit.id());
            if !done {
                stack.push((parent, 0));
            }
        } else {
            stack.pop();
        }
    }

    children.iter_mut().for_each(|(_, set)| {
        for id in std::mem::take(set) {
            match equals.get(&id) {
                Some(&new_id) => set.insert(new_id),
                None => set.insert(id),
            };
        }
    });
    children
        .into_iter()
        .map(|(id, children)| {
            (
                id,
                children
                    .into_iter()
                    .map(|id| repo.find_commit(id).unwrap())
                    .collect(),
            )
        })
        .collect()
}
