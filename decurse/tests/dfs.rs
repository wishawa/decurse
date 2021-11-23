use decurse::decurse;

// #[test]
// fn test_dfs() {
// 	struct Node<T> {
// 		value: T,
// 		children: Vec<Node<T>>
// 	}
// 	#[decurse]
// 	fn dfs<'a, T: PartialEq>(root: &'a Node<T>, find: &T) -> bool {
// 		if &root.value == find {
// 			true
// 		} else {
// 			for child in root.children.iter() {
// 				if dfs(child, find) {
// 					return true;
// 				}
// 			}
// 			false
// 		}
// 	}
// 	let tree = Node {
// 		value: 5,
// 		children: vec![
// 			Node {
// 				value: 3,
// 				children: vec![
// 					Node {
// 						value: 0,
// 						children: vec![]
// 					},
// 					Node {
// 						value: 2,
// 						children: vec![]
// 					}
// 				]
// 			},
// 			Node {
// 				value: 7,
// 				children: vec![]
// 			},
// 			Node {
// 				value: 14,
// 				children: vec![
// 					Node {
// 						value: 16,
// 						children: vec![
// 							Node {
// 								value: 5,
// 								children: vec![]
// 							},
// 							Node {
// 								value: 4,
// 								children: vec![]
// 							}
// 						]
// 					},
// 					Node {
// 						value: 2,
// 						children: vec![]
// 					}
// 				]
// 			}
// 		]
// 	};
// 	assert_eq!(dfs(&tree, &3), true);
// 	assert_eq!(dfs(&tree, &50), false);
// }
