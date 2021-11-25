#[derive(Clone, Copy, Debug)]
enum Tile {
	Unvisited,
	Visited,
	Impassable,
}

type World = Vec<Vec<Tile>>;

// Mark all the reachable tile as Visited.
// Decurse unsound supports borrow.
// However, if safety is your priority, stick to sound version: see dfs_paint example.
// ↓↓ Try removing this, you will get stack overflow.
#[decurse::decurse_unsound]
fn dfs_paint(world: &mut World, (x, y): (isize, isize)) {
	let here = &mut world[x as usize][y as usize];
	*here = Tile::Visited;
	const CHANGE: [isize; 3] = [1, 0, -1];
	for di in &CHANGE {
		let i = x + *di;
		if (i >= world.len() as isize) || (i < 0) {
			continue;
		}
		for dj in &CHANGE {
			let j = y + *dj;
			if (j >= world[i as usize].len() as isize) || (j < 0) {
				continue;
			}
			let neighbor = &mut world[i as usize][j as usize];
			match neighbor {
				Tile::Unvisited => dfs_paint(world, (i, j)),
				_ => {}
			}
		}
	}
}

fn main() {
	const SIZE: usize = 1000;
	const HALF: usize = SIZE / 2;

	// Start off with an empty unvisited world.
	let mut world = vec![vec![Tile::Unvisited; SIZE]; SIZE];

	// Create a wall of Impassable in the middle.
	world[HALF].fill(Tile::Impassable);

	// Paint from (0, 0).
	dfs_paint(&mut world, (0, 0));

	// Expect the half above the wall to be all visited.
	let first_half_visited = world[..HALF]
		.iter()
		.map(|row| {
			row.iter().all(|v| match v {
				Tile::Visited => true,
				_ => false,
			})
		})
		.all(|v| v);
	assert_eq!(first_half_visited, true);

	// Expect the half below the wall to be all unvisited.
	let last_half_unvisited = world[(HALF + 1)..]
		.iter()
		.map(|row| {
			row.iter().all(|v| match v {
				Tile::Unvisited => true,
				_ => false,
			})
		})
		.all(|v| v);
	assert_eq!(last_half_unvisited, true);
}
