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

type State = Vec<ElementId>;

#[derive(Debug, Default)]
struct StateQueue {
    queue: VecDeque<State>,
    queued: HashSet<Vec<ElementId>>,
}

impl StateQueue {
    fn push(&mut self, state: State) {
        let mut key = state.clone();
        key.sort_unstable();
        if self.queued.insert(key) {
            self.queue.push_back(state);
        }
    }

    fn pop(&mut self) -> Option<State> {
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

    let mut init_state = ["Water", "Fire", "Wind", "Earth"]
        .map(|s| db.element_id(s.to_owned()))
        .to_vec();
    for element in config.elements {
        init_state.push(db.element_id(element));
    }

    let mut recipes_found = HashMap::new();
    let mut queue = StateQueue::default();
    queue.push(init_state.clone());

    while let Some(state) = queue.pop() {
        for i in 0..state.len() {
            for j in i..state.len() {
                let output = db.combine(
                    state[i],
                    state[j],
                    |e| eprintln!("API Error: {}. Retrying...", e)
                );

                if state.contains(&output) || db.element_name(output) == "Nothing" {
                    continue;
                }

                let mut child = state.clone();
                child.push(output);
                
                if *recipes_found.entry(output).or_insert(child.len()) == child.len() {
                    println!("{}", db.element_name(output));
                    print_recipe(&mut db, &init_state, &child);
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
}

fn print_recipe(db: &mut ElementDb, init: &[ElementId], state: &[ElementId]) {
    let mut steps = Vec::new();
    for i in 0..state.len() {
        let elements = &state[..i];
        let target = state[i];
        if !init.contains(&target) {
            let pair = find_pair(db, elements, target);
            steps.push((pair, target));
        }
    }
    for ((left, right), output) in steps {
        let output = db.element_name(output);
        let left = db.element_name(left);
        let right = db.element_name(right);
        println!("{} + {} -> {}", left, right, output);
    }
}

fn find_pair(db: &mut ElementDb, elements: &[ElementId], target: ElementId) -> (ElementId, ElementId) {
    for i in 0..elements.len() {
        for j in i..elements.len() {
            let output = db.combine(
                elements[i],
                elements[j],
                |e| eprintln!("API Error: {}. Retrying...", e)
            );
            if output == target {
                return (elements[i], elements[j]);
            }
        }
    }
    panic!()
}
