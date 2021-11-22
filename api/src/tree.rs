use std::slice::Iter;

#[derive(PartialEq, Debug)]
pub struct SubTree<T>
where
    T: Copy,
{
    pub val: T,
    pub children: Vec<SubTree<T>>,
}

impl<T: Copy> SubTree<T> {
    fn new(val: T, children: Vec<SubTree<T>>) -> SubTree<T> {
        SubTree { val, children }
    }
}

#[derive(PartialEq, Debug)]
pub struct Tree<T>
where
    T: Copy,
{
    pub children: Vec<SubTree<T>>,
}

impl<T: Copy> Tree<T> {
    pub fn new(children: Vec<SubTree<T>>) -> Tree<T> {
        Tree { children }
    }
}

pub fn convert<T>(input: &Vec<(usize, T)>) -> Tree<T>
where
    T: Copy,
{
    fn make_tree<T: Copy>(it: &mut Iter<(usize, T)>, indent: usize) -> Vec<SubTree<T>> {
        let mut children = vec![];
        let mut it = it.clone();
        while let Some(&(ind, val)) = it.next() {
            if ind < indent {
                break;
            }
            if ind == indent {
                let subtree_children = make_tree(&mut it, ind + 1);
                children.push(SubTree::new(val, subtree_children));
            }
        }
        return children;
    }

    let mut it = input.iter();
    let children = make_tree(&mut it, 0);

    Tree::new(children)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convert_empty_returns_empty() {
        assert_eq!(convert::<u32>(&vec![]), Tree { children: vec![] });
    }

    #[test]
    fn convert_singleton_returns_something() {
        let input = vec![(0, 'a')];
        let expected = Tree::new(vec![SubTree::new('a', vec![])]);
        assert_eq!(convert(&input), expected);
    }

    #[test]
    fn convert_something_returns_something() {
        let input = vec![
            (0, 'a'),
            (1, 'b'),
            (2, 'c'),
            (1, 'd'),
            (2, 'e'),
            (2, 'f'),
            (0, 'g'),
            (1, 'h'),
            (1, 'i'),
            (0, 'j'),
        ];
        let expected = Tree::new(vec![
            SubTree::new(
                'a',
                vec![
                    SubTree::new('b', vec![SubTree::new('c', vec![])]),
                    SubTree::new(
                        'd',
                        vec![SubTree::new('e', vec![]), SubTree::new('f', vec![])],
                    ),
                ],
            ),
            SubTree::new(
                'g',
                vec![SubTree::new('h', vec![]), SubTree::new('i', vec![])],
            ),
            SubTree::new('j', vec![]),
        ]);
        assert_eq!(convert(&input), expected);
    }
}
