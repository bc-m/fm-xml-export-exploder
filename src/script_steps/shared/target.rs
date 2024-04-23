#[derive(Debug, Clone)]
pub enum Target {
    Field {
        table_occurrence: String,
        name: String,
        repetition: u32,
    },
    Variable {
        name: String,
        repetition: u32,
    },
}

impl Target {
    pub fn new_variable(name: String) -> Self {
        Target::Variable {
            name,
            repetition: 1,
        }
    }

    pub fn set_repetition(&mut self, rep: &str) {
        match self {
            Target::Field { repetition, .. } => *repetition = rep.parse().unwrap(),
            Target::Variable { repetition, .. } => *repetition = rep.parse().unwrap(),
        }
    }
    pub fn set_table_occurrence(&mut self, table_occurrence: String) {
        match self {
            Target::Field {
                table_occurrence: to,
                ..
            } => *to = table_occurrence,
            _ => {}
        }
    }
}

impl std::fmt::Display for Target {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Target::Field {
                table_occurrence,
                name,
                repetition,
            } => {
                if *repetition != 1 {
                    write!(f, "{}::{}[{}]", table_occurrence, name, repetition)
                } else {
                    write!(f, "{}::{}", table_occurrence, name)
                }
            }
            Target::Variable { name, repetition } => {
                if *repetition != 1 {
                    write!(f, "{}[{}]", name, repetition)
                } else {
                    write!(f, "{}", name)
                }
            }
        }
    }
}
