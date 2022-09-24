use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::fs::write;
use semver::{Version, VersionReq, Op};
use toml_edit::{Document, Item};
use bat::PrettyPrinter;

fn main() {
    let local_filename = std::env::var("LOCAL").expect("did not have $LOCAL environment variable");
    let remote_filename = std::env::var("REMOTE").expect("did not have $REMOTE environment variable");
    let merged_filename = std::env::var("MERGED").expect("did not have $MERGED environment variable");

    let local_content = std::fs::read_to_string(local_filename).expect("Could not read local file");
    let remote_content = std::fs::read_to_string(remote_filename).expect("Could not read remote file");

    let result = merge(&local_content, &remote_content);

    write(merged_filename, result.as_bytes()).expect("Failed to write");

    PrettyPrinter::new()
        .input_from_bytes(result.as_bytes())
        .language("toml")
        .print()
        .unwrap();
}

pub fn merge(local: &str, remote: &str) -> String {
    let local_deps = extract_deps(local);
    let remote_deps = extract_deps(remote);

    let a: Vec<_> = local_deps.keys().collect();
    let b: Vec<_> = remote_deps.keys().collect();
    let all = [a, b].concat();

    let mut max = BTreeMap::new();
    for dep in all {
        let local = local_deps.get(dep);
        let remote = remote_deps.get(dep);

        let dep = dep.clone();
        match (local, remote) {
            (None, None) => unreachable!("Found dependency that is neither local nor remote? {dep}"),
            (None, Some(remote)) => {
                max.insert(dep, remote.clone());
            },
            (Some(local), None) => {
                max.insert(dep, local.clone());
            },
            (Some(local), Some(remote)) => {
                if local.version > remote.version {
                    max.insert(dep, local.clone());
                } else {
                    max.insert(dep, remote.clone());
                }
            }
        }
    }

    let mut final_toml = local.parse::<Document>().expect("foo?");
    replace_deps(&mut final_toml, max);
    
    final_toml.to_string()
}

fn replace_deps(toml: &mut toml_edit::Document, deps: BTreeMap<String, Dependency>) {
    let dependencies = &mut toml["dependencies"];

    for (name, dep) in deps {
        dependencies[&name] = dep.toml_item;
    }
}

fn extract_deps(raw: &str) -> BTreeMap<String, Dependency> {
    let remote_toml = raw.parse::<Document>().expect("foo?");
    let mut deps = BTreeMap::new();
    for (name, item) in remote_toml.as_table().get("dependencies").unwrap().as_table().unwrap().iter() {
        let dep = parse_dependency(&name, &item);
        deps.insert(dep.name.clone(), dep);
    }
    deps
}

fn parse_dependency(name: &str, item: &Item) -> Dependency {
    let version = match item {
        Item::Value(toml_edit::Value::String(version)) => version.clone().into_value(),
        Item::Value(toml_edit::Value::InlineTable(table)) => table.get("version").unwrap().as_str().unwrap().to_string(),
        _ => todo!("Random goo"),
    };

    let version = if let Ok(v) = Version::parse(&version) {
        Ver::Exact(v)
    } else {
        VersionReq::parse(&version).map(Ver::Range).expect("Should have been a VersionReq")
    };

    Dependency { name: name.to_string(), version, toml_item: item.clone() }
}

#[derive(Debug, Clone, Eq)]
enum Ver {
    Exact(Version),
    Range(VersionReq),
}

impl PartialEq for Ver {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Exact(l0), Self::Exact(r0)) => l0 == r0,
            (Self::Range(l0), Self::Range(r0)) => l0 == r0,
            _ => false
        }
    }
}

impl PartialOrd for Ver {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (Ver::Exact(v1), Ver::Exact(v2)) => v1.partial_cmp(v2),
            (Ver::Exact(v), Ver::Range(range)) => {
                if range.matches(v) {
                    Some(Ordering::Greater)
                } else {
                    Some(Ordering::Less)
                }
            },
            (Ver::Range(range), Ver::Exact(v)) => {
                if range.matches(v) {
                    Some(Ordering::Less)
                } else {
                    Some(Ordering::Greater)
                }
            },
            (Ver::Range(this), Ver::Range(other)) => {
                if this.comparators.len() != 1 || other.comparators.len() != 1 {
                    // 🤷 no idea what to do in this case
                    return None;
                }

                let this = &this.comparators[0];
                let other = &other.comparators[0];

                if this.op != Op::Caret || other.op != Op::Caret {
                    // 🤷 no idea what to do in this case
                    return None;
                }

                match this.major.cmp(&other.major) {
                    Ordering::Equal => {},
                    other => return Some(other),
                };

                match this.minor.unwrap_or(0).cmp(&other.minor.unwrap_or(0)) {
                    Ordering::Equal => {},
                    other => return Some(other),
                };

                match this.patch.unwrap_or(0).cmp(&other.patch.unwrap_or(0)) {
                    Ordering::Equal => {},
                    other => return Some(other),
                };

                // 🤷 no idea what to do in this case
                None
            }
        }
    }
}


#[derive(Debug, Clone)]
struct Dependency {
    name: String,
    version: Ver,
    toml_item: Item,
}
