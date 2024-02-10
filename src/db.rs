use std::collections::HashMap;
use std::time::Duration;
use std::path::Path;
use std::fs::File;
use std::io::BufReader;
use std::io::BufWriter;

use indexmap::IndexSet;
use serde::Deserialize;
use tempfile::NamedTempFile;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ElementId(u32);

pub struct ElementDb {
    elements: IndexSet<String>,
    database: HashMap<(ElementId, ElementId), ElementId>,
}

#[allow(unused)]
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct IcResult {
    pub result: String,
    pub emoji: String,
    pub is_new: bool,
}

const TIMEOUT: Duration = Duration::from_secs(3);

type Derivations = HashMap<String, Vec<(String, String)>>;

impl ElementDb {
    pub fn new() -> Self {
        Self {
            elements: IndexSet::new(),
            database: HashMap::new(),
        }
    }

    pub fn load(path: &Path) -> Result<Self, std::io::Error> {
        let reader = BufReader::new(File::open(path)?);
        let derivations = serde_json::from_reader::<_, Derivations>(reader)?;
        let mut this = Self::new();
        for (output_name, pairs) in derivations {
            let output = this.element_id(output_name);
            for (a_name, b_name) in pairs {
                let a = this.element_id(a_name);
                let b = this.element_id(b_name);
                let db_key = (a.min(b), a.max(b));
                this.database.insert(db_key, output);
            }
        }
        Ok(this)
    }

    pub fn save(&mut self, path: &Path) {
        let mut derivations = Derivations::new();
        for (&(a, b), &output) in &self.database {
            let a_name = self.element_name(a).to_owned();
            let b_name = self.element_name(b).to_owned();
            let output_name = self.element_name(output).to_owned();
            derivations.entry(output_name)
                .or_default()
                .push((a_name, b_name));
        }

        let dir = path.parent().unwrap();
        let mut file = NamedTempFile::new_in(dir).unwrap();
        let writer = BufWriter::new(&mut file);
        serde_json::to_writer(writer, &derivations).unwrap();
        file.persist(path).unwrap();
    }

    pub fn element_id(&mut self, element: String) -> ElementId {
        match self.elements.get_index_of(&element) {
            Some(index) => ElementId(index as u32),
            None => {
                let index = self.elements.len();
                self.elements.insert(element);
                ElementId(index as u32)
            }
        }
    }

    pub fn element_name(&self, id: ElementId) -> &str {
        self.elements.get_index(id.0 as usize).unwrap()
    }

    pub fn combine(&mut self, a: ElementId, b: ElementId, mut on_error: impl FnMut(ureq::Error)) -> ElementId {
        let db_key = (a.min(b), a.max(b));
        if let Some(&output) = self.database.get(&db_key) {
            return output;
        }

        let a_name = self.element_name(a);
        let b_name = self.element_name(b);
        loop {
            let request = ureq::get("https://neal.fun/api/infinite-craft/pair")
                .set("Referer", "https://neal.fun/infinite-craft/")
                .set("User-Agent", "curl/7.54.1")
                .query("first", a_name.min(b_name))
                .query("second", a_name.max(b_name));
            let response = match request.call() {
                Ok(response) => response,
                Err(error) => {
                    on_error(error);
                    std::thread::sleep(TIMEOUT);
                    continue;
                }
            };
            let result = match response.into_json::<IcResult>() {
                Ok(result) => result,
                Err(error) => {
                    on_error(error.into());
                    std::thread::sleep(TIMEOUT);
                    continue;
                }
            };
            let output = self.element_id(result.result);
            self.database.insert(db_key, output);
            return output;
        }
    }
}
