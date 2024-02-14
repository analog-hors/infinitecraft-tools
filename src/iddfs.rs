use std::collections::HashMap;
use std::collections::hash_map;
use std::path::Path;
use std::time::{Instant, Duration};

use clap::Args;
use indexmap::IndexMap;

use crate::db::{ElementDb, ElementId};

#[derive(Debug, Args)]
/// Do an Iterative Deepening Depth-First Search of the state space to find optimal routes
pub struct Config {
    /// Additional elements to add to the initial state
    #[arg(short, long, num_args(0..))]
    elements: Vec<String>,
}

const SAVE_INTERVAL: Duration = Duration::from_secs(60);

type State = IndexMap<ElementId, Option<((ElementId, ElementId), u32)>>;

pub fn run(config: Config) {
    let db_path = Path::new("db.json");
    let mut db = match ElementDb::load(db_path) {
        Ok(db) => db,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => ElementDb::new(),
        Err(err) => {
            eprintln!("error while loading db: {}", err);
            return
        }
    };
    let mut last_save = Instant::now();

    let mut base = ["Water", "Fire", "Wind", "Earth"]
        .map(|s| db.element_id(s.to_owned()))
        .to_vec();
    for element in config.elements {
        base.push(db.element_id(element));
    }

    let mut state = base.iter().map(|&e| (e, None)).collect();
    let mut banned = base.iter().map(|&e| (e, 1)).collect();
    let mut recipe_depths = HashMap::new();
    for depth in 1.. {
        iddfs(&mut db, &mut state, &mut banned, base.len() as u32, depth, &mut |db, state| {
            if last_save.elapsed() >= SAVE_INTERVAL {
                eprintln!("Saving DB...");
                db.save(db_path);
                last_save = Instant::now();
            }

            let (&element, _) = state.last().unwrap();
            if *recipe_depths.entry(element).or_insert(depth) != depth {
                return;
            }

            println!("{}", db.element_name(element));
            for (&output, &step) in state.iter() {
                if let Some(((a, b), _)) = step {
                    let output = db.element_name(output);
                    let a = db.element_name(a);
                    let b = db.element_name(b);
                    println!("{} + {} -> {}", a, b, output);
                }
            }
            println!();
        });
    }
}

fn iddfs(
    db: &mut ElementDb,
    state: &mut State,
    banned: &mut HashMap<ElementId, u32>,
    used: u32,
    depth: u32,
    on_recipe: &mut impl FnMut(&mut ElementDb, &State)
) {
    if depth == 0 {
        on_recipe(db, state);
        return;
    }

    if state.len() as u32 - used > depth + 1 {
        return;
    }

    edges(db, state, |db, state, output, (a, b)| {
        let banned_entry = banned.entry(output).or_default();
        *banned_entry += 1;
        if *banned_entry > 1 {
            return;
        }

        let mut used = used;
        if let Some((_, usages)) = state.get_mut(&a).unwrap() {
            *usages += 1;
            if *usages == 1 {
                used += 1;
            }
        }
        if let Some((_, usages)) = state.get_mut(&b).unwrap() {
            *usages += 1;
            if *usages == 1 {
                used += 1;
            }
        }
        
        state.insert(output, Some(((a, b), 0)));
        iddfs(db, state, banned, used, depth - 1, on_recipe);
        state.pop();

        if let Some((_, usages)) = state.get_mut(&a).unwrap() {
            *usages -= 1;
        }
        if let Some((_, usages)) = state.get_mut(&b).unwrap() {
            *usages -= 1;
        }
    });
    edges(db, state, |_, _, output, _| {
        if let hash_map::Entry::Occupied(mut entry) = banned.entry(output) {
            match entry.get_mut() {
                1 => {
                    entry.remove();
                }
                n => *n -= 1,
            }
        }
    });
}

fn edges(
    db: &mut ElementDb,
    state: &mut State,
    mut on_edge: impl FnMut(&mut ElementDb, &mut State, ElementId, (ElementId, ElementId))
) {
    let next = |i, j| if i < j { (i + 1, j) } else { (0, j + 1) };
    let (mut i, mut j) = match state.last().unwrap() {
        (_, Some(((a, b), _))) => {
            let a_index = state.get_index_of(a).unwrap();
            let b_index = state.get_index_of(b).unwrap();
            next(a_index.min(b_index), a_index.max(b_index))
        }
        (_, None) => (0, 0),
    };
    while j < state.len() {
        let (&a, _) = state.get_index(i).unwrap();
        let (&b, _) = state.get_index(j).unwrap();
        let output = db.combine(a, b, on_api_error);
        if db.element_name(output) != "Nothing" {
            on_edge(db, state, output, (a, b));
        }

        (i, j) = next(i, j);
    }
}

fn on_api_error(error: ureq::Error) {
    eprintln!("API Error: {}. Retrying...", error);
}
