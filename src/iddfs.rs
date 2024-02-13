use std::collections::HashMap;
use std::collections::hash_map;
use std::path::Path;
use std::time::{Instant, Duration};

use clap::Args;
use indexmap::IndexMap;

use crate::db::{ElementDb, ElementId};

#[derive(Debug, Args)]
/// Do a Breadth-First Search of the state space to find optimal routes
pub struct Config {
    /// Additional elements to add to the initial state
    #[arg(short, long, num_args(0..))]
    elements: Vec<String>,
}

const SAVE_INTERVAL: Duration = Duration::from_secs(60);

type State = IndexMap<ElementId, Option<(ElementId, ElementId)>>;

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
    let mut banned = HashMap::new();
    let mut recipe_depths = HashMap::new();
    for depth in 1.. {
        iddfs(&mut db, &mut state, &mut banned, depth, &mut |db, state| {
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
            for (&output, &derivation) in state.iter() {
                if let Some((a, b)) = derivation {
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
    depth: u32,
    on_recipe: &mut impl FnMut(&mut ElementDb, &State)
) {
    if depth == 0 {
        on_recipe(db, state);
        return;
    }

    edges(db, state, |db, state, output, derivation| {
        let banned_entry = banned.entry(output).or_default();
        *banned_entry += 1;
        if *banned_entry > 1 {
            return;
        }
        if let indexmap::map::Entry::Vacant(entry) = state.entry(output) {
            entry.insert(Some(derivation));
            iddfs(db, state, banned, depth - 1, on_recipe);
            state.pop();
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
        (_, Some((a, b))) => {
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
