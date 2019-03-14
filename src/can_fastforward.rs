// Fast-forward if all commits are ancestors or descendants of each other.
// Forms a complete-graph of comparisons.
// i.e. (n*(n-1))/2 comparisons.
//   2 commits: 1 comparison
//   3 commits: 3 comparisons
//   4 commits: 6 comparisons
//   ...
pub fn can_fastforward<T, F>(nodes: &[T], is_parent: F) -> Option<T>
  where T: Copy,
        T: Eq,
        F: Fn(T, T) -> bool {
  let nodes_len = nodes.len();

  if nodes_len == 1 {
      return Some(nodes[0]);
  }

  let mut most_recent: Option<T> = None;

  for i in 0..nodes_len {
    for j in (i+1)..nodes_len {
        let a = nodes[i];
        let b = nodes[j];

        let (old, new) = if is_parent(a, b) {
            (a, b)
        } else if is_parent(b, a) {
            (b, a)
        } else {
            return None;
        };

        if let Some(r) = most_recent {
            if r == old {
                most_recent = Some(new);
            }
        } else {
            most_recent = Some(new);
        }
    }
  }

  most_recent
}

#[cfg(test)]
mod tests {
    use super::*;
    fn can_fastforward_helper_is_parent(parent: u8, child: u8, adjacency: &[[u8; 2]]) -> bool {
        if parent == child { return true }

        for [a, b] in adjacency {
            if *a != parent { continue }

            if can_fastforward_helper_is_parent(*b, child, adjacency) {
                return true
            }
        }

        false
    }

    fn can_fastforward_helper(commits: &[u8], adjacency: &[[u8; 2]]) -> Option<u8> {
        can_fastforward(commits, |parent, child| {
            can_fastforward_helper_is_parent(parent, child, adjacency)
        })
    }

    fn can_fastforward_to(commits: &[u8], adjacency: &[[u8; 2]], to: u8) {
        assert_eq!(can_fastforward_helper(commits, adjacency), Some(to));
    }
    fn can_fastforward_false(commits: &[u8], adjacency: &[[u8; 2]]) {
        assert_eq!(can_fastforward_helper(commits, adjacency).is_some(), false);
    }

    #[test]
    fn can_fastforward_test() {
        // a' = 1
        // b' = 2
        // from = 3

        // a' -> b' -> from
        // Result: fast-forward.
        can_fastforward_to(&[1, 2, 3],
                           &[ [1, 2], [2, 3] ],
                           3);
        can_fastforward_to(&[1, 3],
                           &[ [1, 2], [2, 3] ],
                           3);

        // from -> a' -> b'
        // Result: fast-forward. "from" moves to most recent commit, "b'".
        can_fastforward_to(&[1, 2, 3],
                           &[ [3, 1], [1, 2] ],
                           2);
        can_fastforward_to(&[2, 3],
                           &[ [3, 1], [1, 2] ],
                           2);

        // a' -> from -> b'
        // Result: fast-forward. "from" moves to most recent commit, "b'".
        can_fastforward_to(&[1, 2, 3],
                           &[ [1, 3], [3, 2] ],
                           2);
        can_fastforward_to(&[1, 3],
                           &[ [1, 3], [3, 2] ],
                           // not 2, because we didn't specify it
                           3);

        // from -> a'
        //      \> b'
        // Result: no fast-forward.
        //         from -> a' -> from'
        //              \> b' -/
        can_fastforward_false(&[1, 2, 3],
                              &[ [3, 1], [3, 2] ]);
        
        // a' -> from
        //    \> b'
        // Result: no fast-foward.
        //         a' -> from -> from'
        //            \> b'   -/
        can_fastforward_false(&[1, 2, 3],
                              &[ [1, 3], [1, 2] ]);

        // a'
        // b'
        // (no common commits)
        can_fastforward_false(&[1, 2],
                              &[]);

        // Can fast-forward a single commit (just do nothing).
        can_fastforward_to(&[1],
                           &[],
                           1);

        // Can't fast-forward with nothing
        can_fastforward_false(&[],
                              &[]);

    }
}