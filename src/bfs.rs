use std::collections::{HashMap, HashSet, VecDeque};
use std::path::Path;
use std::time::{Instant, Duration};

use clap::Args;

use crate::db::{ElementDb, ElementId};

#[derive(Debug, Args)]
/// Do a Breadth-First Search of the state space to find optimal routes
pub struct Config {
    /// Additional elements to add to the initial state
    #[arg(short, long, num_args(0..))]
    elements: Vec<String>,
}

#[derive(Debug, Default)]
struct StateQueue {
    queue: VecDeque<Vec<ElementId>>,
    queued: HashSet<Vec<ElementId>>,
}

impl StateQueue {
    fn push(&mut self, state: Vec<ElementId>) {
        let mut key = state.clone();
        key.sort_unstable();
        if self.queued.insert(key) {
            self.queue.push_back(state);
        }
    }

    fn pop(&mut self) -> Option<Vec<ElementId>> {
        let state = self.queue.pop_front()?;
        let mut key = state.clone();
        key.sort_unstable();
        self.queued.remove(&key);
        Some(state)
    }
}

const SAVE_INTERVAL: Duration = Duration::from_secs(60);

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

    let mut recipes_found = HashMap::new();
    let mut queue = StateQueue::default();
    queue.push(Vec::new());

    while let Some(state) = queue.pop() {
        for (a, b) in edges(&base, &state) {
            let output = db.combine(a, b, on_api_error);

            if state.contains(&output) || db.element_name(output) == "Nothing" {
                continue;
            }

            let mut child = state.clone();
            child.push(output);
            
            if *recipes_found.entry(output).or_insert(child.len()) == child.len() {
                println!("{}", db.element_name(output));
                print_recipe(&mut db, &base, &child);
                println!();
            }

            queue.push(child);

            if last_save.elapsed() >= SAVE_INTERVAL {
                eprintln!("Saving DB...");
                db.save(db_path);
                last_save = Instant::now();
            }
        }
    }
}

fn edges<'a>(base: &'a [ElementId], state: &'a [ElementId]) -> impl Iterator<Item=(ElementId, ElementId)> + 'a {
    (0..base.len() + state.len())
        .flat_map(|i| (i..base.len() + state.len()).map(move |j| (i, j)))
        .map(|(i, j)| (
            if i < base.len() { base[i] } else { state[i - base.len()] },
            if j < base.len() { base[j] } else { state[j - base.len()] },
        ))
}

fn print_recipe(db: &mut ElementDb, base: &[ElementId], state: &[ElementId]) {
    for i in 0..state.len() {
        let output = state[i];
        let (a, b) = edges(base, &state[..i])
            .find(|&(a, b)| db.combine(a, b, on_api_error) == output)
            .unwrap();
        let output = db.element_name(output);
        let a = db.element_name(a);
        let b = db.element_name(b);
        println!("{} + {} -> {}", a, b, output);
    }
}

fn on_api_error(error: ureq::Error) {
    eprintln!("API Error: {}. Retrying...", error);
}
