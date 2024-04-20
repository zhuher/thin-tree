use rand::{
    prelude::*,
    rngs::{OsRng, ReseedingRng},
};
use rand_chacha::{rand_core::SeedableRng, ChaCha20Core};
use std::io::{stdout, Write};
#[derive(Debug, Clone)]
enum Node {
    Leaf,
    Branch(Box<(Node, Node)>),
}
impl TryFrom<Node> for (Node, Node) {
    type Error = &'static str;
    fn try_from(value: Node) -> Result<Self, Self::Error> {
        match value {
            Node::Branch(bx) => Ok((bx.0, bx.1)),
            _ => Err("Not a branch"),
        }
    }
}
macro_rules! colour {
    ($colour:expr, $thing:expr, $tint:expr) => {
        if $colour {
            format!("\x1B[3{}m{}\x1B[0m", $tint, $thing)
        } else {
            format!("{}", $thing)
        }
    };
}
macro_rules! clear {
    ($colour:expr) => {
        if $colour {
            format!("\x1B[2J\x1B[1;1H")
        } else {
            String::new()
        }
    };
}
fn check_stats(n: u32, m: u32, fast_rng: bool, sample_size: u32) -> (u32, u32, u32, u32, f64) {
    let mut cur_vals = (0, 0, 0, 0, 0.0);
    let mut leaves_vec = Vec::new();
    for _ in 0..sample_size {
        let tree = generate_tree(true, n, m, fast_rng);
        let leaves = count_leaves(&tree);
        leaves_vec.push(leaves);
        cur_vals.2 = cur_vals.2.max(leaves);
        cur_vals.0 = cur_vals.0.min(leaves);
        if cur_vals.0 == 0 {
            cur_vals.0 = leaves;
        }
    }
    leaves_vec.sort();
    cur_vals.1 = leaves_vec[leaves_vec.len() / 2];
    cur_vals.3 = leaves_vec.iter().sum::<u32>() / leaves_vec.len() as u32;
    cur_vals.4 = (leaves_vec
        .iter()
        .map(|x| (*x as f64 - cur_vals.3 as f64).powi(2))
        .sum::<f64>()
        / leaves_vec.len() as f64)
        .sqrt();
    cur_vals
}
fn get_tree_stats(tree: &Node) -> String {
    format!(
        "Leaves: {}\n\
	 Branches: {}\n\
	 Nodes: {}\n\
	 Generations: {}",
        count_leaves(tree),
        count_branches(tree),
        count_nodes(tree),
        count_generations(tree),
    )
}
fn get_tree_rolls(tree: &Node, generations: u32, colour: bool) -> String {
    format!(
        "Rolls: {}\n{}",
        tree_to_string(tree, colour),
        (0..=generations).fold(String::new(), |mut acc, i| {
            acc.push_str(&format!(
                "Gen {0:^1$}: {2}\n",
                i,
                format!("{generations}").len(),
                get_nodes_at_generation(tree, i, 0, colour)
            ));
            acc
        })
    )
}
fn print_stats_delta(
    prev_vals: (u32, u32, u32, u32, f64),
    cur_vals: (u32, u32, u32, u32, f64),
    colour: bool,
) -> String {
    format!(
        "Tree stats:\
	 \n\tMin = {} ({})\
	 \n\tMedian = {} ({})\
	 \n\tMax = {} ({})\
	 \n\tAverage = {} ({})\
	 \n\tσ = {} ({})",
        match cur_vals.0.cmp(&prev_vals.0) {
            std::cmp::Ordering::Less => colour!(colour, cur_vals.0, 1),
            std::cmp::Ordering::Greater => colour!(colour, cur_vals.0, 2),
            std::cmp::Ordering::Equal => colour!(colour, cur_vals.0, 4),
        },
        match cur_vals.0.cmp(&prev_vals.0) {
            std::cmp::Ordering::Less => colour!(
                colour,
                format!(
                    "↓{}",
                    prev_vals.0.max(cur_vals.0) - prev_vals.0.min(cur_vals.0)
                ),
                1
            ),
            std::cmp::Ordering::Greater => colour!(
                colour,
                format!(
                    "↑{}",
                    prev_vals.0.max(cur_vals.0) - prev_vals.0.min(cur_vals.0)
                ),
                2
            ),
            std::cmp::Ordering::Equal => colour!(
                colour,
                format!(
                    "={}",
                    prev_vals.0.max(cur_vals.0) - prev_vals.0.min(cur_vals.0)
                ),
                4
            ),
        },
        match cur_vals.1.cmp(&prev_vals.1) {
            std::cmp::Ordering::Less => colour!(colour, cur_vals.1, 1),
            std::cmp::Ordering::Greater => colour!(colour, cur_vals.1, 2),
            std::cmp::Ordering::Equal => colour!(colour, cur_vals.1, 4),
        },
        match cur_vals.1.cmp(&prev_vals.1) {
            std::cmp::Ordering::Less => colour!(
                colour,
                format!(
                    "↓{}",
                    prev_vals.1.max(cur_vals.1) - prev_vals.1.min(cur_vals.1)
                ),
                1
            ),
            std::cmp::Ordering::Greater => colour!(
                colour,
                format!(
                    "↑{}",
                    prev_vals.1.max(cur_vals.1) - prev_vals.1.min(cur_vals.1)
                ),
                2
            ),
            std::cmp::Ordering::Equal => colour!(
                colour,
                format!(
                    "={}",
                    prev_vals.1.max(cur_vals.1) - prev_vals.1.min(cur_vals.1)
                ),
                4
            ),
        },
        match cur_vals.2.cmp(&prev_vals.2) {
            std::cmp::Ordering::Less => colour!(colour, cur_vals.2, 1),
            std::cmp::Ordering::Greater => colour!(colour, cur_vals.2, 2),
            std::cmp::Ordering::Equal => colour!(colour, cur_vals.2, 4),
        },
        match cur_vals.2.cmp(&prev_vals.2) {
            std::cmp::Ordering::Less => colour!(
                colour,
                format!(
                    "↓{}",
                    prev_vals.2.max(cur_vals.2) - prev_vals.2.min(cur_vals.2)
                ),
                1
            ),
            std::cmp::Ordering::Greater => colour!(
                colour,
                format!(
                    "↑{}",
                    prev_vals.2.max(cur_vals.2) - prev_vals.2.min(cur_vals.2)
                ),
                2
            ),
            std::cmp::Ordering::Equal => colour!(
                colour,
                format!(
                    "={}",
                    prev_vals.2.max(cur_vals.2) - prev_vals.2.min(cur_vals.2)
                ),
                4
            ),
        },
        match cur_vals.3.cmp(&prev_vals.3) {
            std::cmp::Ordering::Less => colour!(colour, cur_vals.3, 1),
            std::cmp::Ordering::Greater => colour!(colour, cur_vals.3, 2),
            std::cmp::Ordering::Equal => colour!(colour, cur_vals.3, 4),
        },
        match cur_vals.3.cmp(&prev_vals.3) {
            std::cmp::Ordering::Less => colour!(
                colour,
                format!(
                    "↓{}",
                    prev_vals.3.max(cur_vals.3) - prev_vals.3.min(cur_vals.3)
                ),
                1
            ),
            std::cmp::Ordering::Greater => colour!(
                colour,
                format!(
                    "↑{}",
                    prev_vals.3.max(cur_vals.3) - prev_vals.3.min(cur_vals.3)
                ),
                2
            ),
            std::cmp::Ordering::Equal => colour!(
                colour,
                format!(
                    "={}",
                    prev_vals.3.max(cur_vals.3) - prev_vals.3.min(cur_vals.3)
                ),
                4
            ),
        },
        match cur_vals.4.partial_cmp(&prev_vals.4).unwrap() {
            std::cmp::Ordering::Less => colour!(colour, cur_vals.4, 1),
            std::cmp::Ordering::Greater => colour!(colour, cur_vals.4, 2),
            std::cmp::Ordering::Equal => colour!(colour, cur_vals.4, 4),
        },
        match cur_vals.4.partial_cmp(&prev_vals.4).unwrap() {
            std::cmp::Ordering::Less => colour!(
                colour,
                format!(
                    "↓{}",
                    prev_vals.4.max(cur_vals.4) - prev_vals.4.min(cur_vals.4)
                ),
                1
            ),
            std::cmp::Ordering::Greater => colour!(
                colour,
                format!(
                    "↑{}",
                    prev_vals.4.max(cur_vals.4) - prev_vals.4.min(cur_vals.4)
                ),
                2
            ),
            std::cmp::Ordering::Equal => colour!(
                colour,
                format!(
                    "={}",
                    prev_vals.4.max(cur_vals.4) - prev_vals.4.min(cur_vals.4)
                ),
                4
            ),
        },
    )
}
fn main() {
    let mut prev_vals: (u32, u32, u32, u32, f64) = (0, 0, 0, 0, 0.0);
    let mut cur_vals: (u32, u32, u32, u32, f64);
    let mut m = 100;
    let mut n = 50;
    let mut tree: Node = Node::Leaf;
    let mut status = String::new();
    let mut sample_size = 1000;
    let mut fast_rng = true;
    let mut colour = true;
    let mut stdout_lock = stdout().lock();
    'main: loop {
        let mut input = String::new();
        write!(
            stdout_lock,
            "{}\
	     {status}{}\
	     Greetings!\n\
	     Current settings are:\n\t\
	     Branch P: {}({n}/{m});{}\n\
	     What would you like to do?\n\t\
	     1. Change settings\n\t\
	     2. Generate tree\n\t\
	     3. Print tree\n\t\
	     4. Collect stats from P\n\t\
	     5. Write {} samples to file\n\t\
	     6. Write current tree to file\n\t\
	     7. Exit\n> ",
            clear!(colour),
            if !status.is_empty() && !status.ends_with('\n') {
                "\n"
            } else {
                ""
            },
            n as f64 / m as f64,
            if n as f64 / m as f64 > 0.6 {
                colour!(
                    colour,
                    "\nWarning: high P may generate an infinite tree and crash.",
                    1
                )
            } else {
                String::new()
            },
            colour!(colour, sample_size, 4)
        )
        .unwrap();
        stdout_lock.flush().unwrap();
        status = String::new();
        match std::io::stdin().read_line(&mut input) {
            Ok(_) => match input.trim() {
                "1" => 'settings: loop {
                    write!(
                        stdout_lock,
                        "{}\
			 {status}{}\
			 Branch P: {}\n\
			 What would you like to change?\n\t\
			 1. n({})\n\t\
			 2. m({})\n\t\
			 3. Change RNG strategy({})\n\t\
			 4. Change sample size({})\n\t\
			 5. Enable/disable colours\n\t\
			 6. Back\n> ",
                        clear!(colour),
                        if !status.is_empty() && !status.ends_with('\n') {
                            "\n"
                        } else {
                            ""
                        },
                        n as f64 / m as f64,
                        colour!(colour, n, 4),
                        colour!(colour, m, 4),
                        colour!(
                            colour,
                            if fast_rng { "Fast" } else { "Secure" },
                            if fast_rng { 2 } else { 5 }
                        ),
                        colour!(colour, sample_size, 4)
                    )
                    .unwrap();
                    stdout_lock.flush().unwrap();
                    status = String::new();
                    let mut input = String::new();
                    match std::io::stdin().read_line(&mut input) {
                        Ok(_) => match input.trim() {
                            "1" => {
                                write!(stdout_lock, "Enter new n: ").unwrap();
                                stdout_lock.flush().unwrap();
                                let mut input = String::new();
                                match std::io::stdin().read_line(&mut input) {
                                    Ok(_) => match input.trim().parse::<u32>() {
                                        Ok(val) => {
                                            n = val;
                                            status =
                                                colour!(colour, format!("Changed n to {}", n), 2);
                                        }
                                        Err(e) => {
                                            status = colour!(
                                                colour,
                                                format!("Error parsing input: {}", e),
                                                1
                                            );
                                        }
                                    },
                                    Err(e) => {
                                        status = colour!(
                                            colour,
                                            format!("Error reading input: {}", e),
                                            1
                                        );
                                    }
                                }
                            }
                            "2" => {
                                write!(stdout_lock, "Enter new m: ").unwrap();
                                stdout_lock.flush().unwrap();
                                let mut input = String::new();
                                match std::io::stdin().read_line(&mut input) {
                                    Ok(_) => match input.trim().parse::<u32>() {
                                        Ok(val) => {
                                            m = val;
                                            status =
                                                colour!(colour, format!("Changed m to {}", m), 2);
                                        }
                                        Err(e) => {
                                            status = colour!(
                                                colour,
                                                format!("Error parsing input: {}", e),
                                                1
                                            );
                                        }
                                    },
                                    Err(e) => {
                                        status = colour!(
                                            colour,
                                            format!("Error reading input: {}", e),
                                            1
                                        );
                                    }
                                }
                            }
                            "3" => {
                                fast_rng = !fast_rng;
                                status = format!(
                                    "Changed RNG strategy to {}",
                                    if fast_rng {
                                        colour!(colour, "Fast", 2)
                                    } else {
                                        colour!(colour, "Secure", 5)
                                    }
                                );
                            }
                            "4" => {
                                write!(stdout_lock, "Enter new sample size: ").unwrap();
                                stdout_lock.flush().unwrap();
                                let mut input = String::new();
                                match std::io::stdin().read_line(&mut input) {
                                    Ok(_) => match input.trim().parse::<u32>() {
                                        Ok(val) => {
                                            sample_size = val;
                                            status = format!(
                                                "Changed sample size to {}\n{}",
                                                colour!(colour, sample_size, 4),
                                                if sample_size > 100000 {
                                                    colour!(
                                                        colour,
                                                        "Warning: sample size is very large",
                                                        1
                                                    )
                                                } else {
                                                    String::new()
                                                }
                                            );
                                        }
                                        Err(e) => {
                                            status = colour!(
                                                colour,
                                                format!("Error parsing input: {}", e),
                                                1
                                            );
                                        }
                                    },
                                    Err(e) => {
                                        status = colour!(
                                            colour,
                                            format!("Error reading input: {}", e),
                                            1
                                        );
                                    }
                                }
                            }
                            "5" => {
                                colour = !colour;
                                status = colour!(
                                    colour,
                                    format!(
                                        "Colours are now {}",
                                        if colour { "enabled" } else { "disabled" },
                                    ),
                                    2
                                );
                            }
                            "6" => {
                                break 'settings;
                            }
                            _ => {
                                status = colour!(colour, "Invalid input", 1);
                            }
                        },
                        Err(e) => {
                            status = colour!(colour, format!("Error reading input: {}", e), 1);
                        }
                    }
                },
                "2" => {
                    tree = generate_tree(true, n, m, fast_rng);
                    status = format!(
                        "{}\n{}",
                        colour!(colour, "Tree generated", 2),
                        get_tree_stats(&tree)
                    );
                }
                "3" => {
                    status = format!(
                        "{}{}{}",
                        print_tree(&tree, colour, Vec::new()),
                        get_tree_rolls(&tree, count_generations(&tree), colour),
                        get_tree_stats(&tree)
                    );
                }
                "4" => {
                    cur_vals = check_stats(n, m, fast_rng, sample_size);
                    status = format!(
                        "Generated {} samples:\n{}",
                        colour!(colour, sample_size, 4),
                        print_stats_delta(prev_vals, cur_vals, colour)
                    );
                    prev_vals = cur_vals;
                }
                "5" => {
                    let default_filename = format!("{n}-{m}-x{sample_size}");
                    let mut input = String::new();
                    write!(
                        stdout_lock,
                        "Enter filename without extension[{default_filename}]: "
                    )
                    .unwrap();
                    stdout_lock.flush().unwrap();
                    match std::io::stdin().read_line(&mut input) {
                        Ok(_) => match std::fs::write(
                            format!(
                                "{}.csv",
                                if input.trim().is_empty() {
                                    &default_filename
                                } else {
                                    input.trim()
                                }
                            ),
                            format!(
                                "0,leaves,branches,nodes,generations,rolls\n{}",
                                (1..=sample_size).fold(String::new(), |mut acc, i| {
                                    let t = generate_tree(true, n, m, fast_rng);
                                    acc.push_str(&format!(
                                        "{i},{},{},{},{},{}\n",
                                        count_leaves(&t),
                                        count_branches(&t),
                                        count_nodes(&t),
                                        count_generations(&t),
                                        tree_to_string(&t, false)
                                    ));
                                    acc
                                })
                            ),
                        ) {
                            Ok(_) => {
                                status = format!(
                                    "Wrote {} samples to file {}",
                                    colour!(colour, sample_size, 4),
                                    colour!(
                                        colour,
                                        if input.trim().is_empty() {
                                            &default_filename
                                        } else {
                                            input.trim()
                                        },
                                        4
                                    )
                                );
                            }
                            Err(e) => {
                                status = colour!(colour, format!("Error writing file: {}", e), 1);
                            }
                        },
                        Err(e) => {
                            status = colour!(colour, format!("Error reading input: {}", e), 1);
                        }
                    }
                }
                "6" => {
                    let default_filename = format!(
                        "{}-{}-{}-{}",
                        count_leaves(&tree),
                        count_branches(&tree),
                        count_nodes(&tree),
                        count_generations(&tree)
                    );
                    let mut input = String::new();
                    write!(
                        stdout_lock,
                        "Enter filename without extension[{default_filename}]: "
                    )
                    .unwrap();
                    stdout_lock.flush().unwrap();
                    match std::io::stdin().read_line(&mut input) {
                        Ok(_) => match std::fs::write(
                            format!(
                                "{}.csv",
                                if input.trim().is_empty() {
                                    &default_filename
                                } else {
                                    input.trim()
                                }
                            ),
                            format!(
                                "{},{},{},{},{}\n",
                                count_leaves(&tree),
                                count_branches(&tree),
                                count_nodes(&tree),
                                count_generations(&tree),
                                tree_to_string(&tree, false),
                            ),
                        ) {
                            Ok(_) => {
                                status = format!(
                                    "Wrote current tree to file {}",
                                    colour!(
                                        colour,
                                        if input.trim().is_empty() {
                                            &default_filename
                                        } else {
                                            input.trim()
                                        },
                                        4
                                    )
                                );
                            }
                            Err(e) => {
                                status = colour!(colour, format!("Error writing file: {}", e), 1);
                            }
                        },
                        Err(e) => {
                            status = colour!(colour, format!("Error reading input: {}", e), 1);
                        }
                    }
                }
                "7" => {
                    break 'main;
                }
                _ => {
                    status = colour!(colour, "Invalid input", 1);
                }
            },
            Err(e) => {
                status = colour!(colour, format!("Error reading input: {}", e), 1);
            }
        }
    }
}
fn print_tree(tree: &Node, last: bool, mut branches: Vec<u32>) -> String {
    let generations = *branches.last().unwrap_or(&0);
    let mut res = String::new();
    if last {
        branches.pop();
    }
    for i in 1..generations {
        res.push_str(if branches.contains(&i) { "║" } else { " " });
    }
    match tree {
        Node::Leaf => {
            res.push_str(if last { "╚Leaf\n" } else { "╠Leaf\n" });
            res
        }

        Node::Branch(bx) => {
            res.push_str(if generations == 0 {
                "Root\n"
            } else if last {
                "╚Branch\n"
            } else {
                "╠Branch\n"
            });
            let (mut new_branches1, mut new_branches2) = (branches.clone(), branches.clone());
            new_branches1.push(generations + 1);
            res.push_str(&print_tree(&bx.0, false, new_branches1));
            new_branches2.push(generations + 1);
            res.push_str(&print_tree(&bx.1, true, new_branches2));
            res
        }
    }
}

fn generate_tree(root: bool, n: u32, m: u32, fast_rng: bool) -> Node {
    let branch_chance = if fast_rng {
        |n, m| rand::thread_rng().gen_range(0..m) < n //Fast RNG
    } else {
        |n, m| ReseedingRng::new(ChaCha20Core::from_entropy(), 4, OsRng).gen_range(0..m) < n
        // CSRNG
    };
    if root {
        Node::Branch(Box::from((
            Node::Branch(Box::from((
                generate_tree(false, n, m, fast_rng),
                generate_tree(false, n, m, fast_rng),
            ))),
            Node::Branch(Box::from((
                generate_tree(false, n, m, fast_rng),
                generate_tree(false, n, m, fast_rng),
            ))),
        )))
    } else if branch_chance(m - n, m) {
        Node::Leaf
    } else {
        Node::Branch(Box::from((
            generate_tree(false, n, m, fast_rng),
            if branch_chance(n, m) {
                generate_tree(false, n, m, fast_rng)
            } else {
                Node::Leaf
            },
        )))
    }
}
fn count_leaves(tree: &Node) -> u32 {
    match tree {
        Node::Leaf => 1,
        Node::Branch(bx) => count_leaves(&bx.0) + count_leaves(&bx.1),
    }
}
fn count_branches(tree: &Node) -> u32 {
    match tree {
        Node::Leaf => 0,
        Node::Branch(bx) => 1 + count_branches(&bx.0) + count_branches(&bx.1),
    }
}
fn count_nodes(tree: &Node) -> u32 {
    match tree {
        Node::Leaf => 1,
        Node::Branch(bx) => 1 + count_nodes(&bx.0) + count_nodes(&bx.1),
    }
}
fn count_generations(tree: &Node) -> u32 {
    match tree {
        Node::Leaf => 0,
        Node::Branch(bx) => 1 + count_generations(&bx.0).max(count_generations(&bx.1)),
    }
}
fn get_nodes_at_generation(tree: &Node, seek_gen: u32, gen: u32, colour: bool) -> String {
    if gen == seek_gen {
        match tree {
            Node::Leaf => colour!(colour, "0", gen % 8),
            Node::Branch(_) => colour!(colour, "1", gen % 8),
        }
    } else {
        match tree {
            Node::Leaf => String::from(""),
            Node::Branch(bx) => {
                get_nodes_at_generation(&bx.0, seek_gen, gen + 1, colour)
                    + &get_nodes_at_generation(&bx.1, seek_gen, gen + 1, colour)
            }
        }
    }
}

fn tree_to_string(tree: &Node, colour: bool) -> String {
    (2..=count_generations(tree)).fold(String::new(), |mut acc, i| {
        acc.push_str(&get_nodes_at_generation(tree, i, 0, colour));
        acc
    })
}
