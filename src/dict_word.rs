use std::default::Default;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DictWord {
    pub word_header: DictWordHeader,
    pub definition_md: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DictWordHeader {
    pub dict_label: String,
    pub word: String,
    pub summary: String,
    pub grammar: String,
    pub inflections: Vec<String>,
}

impl DictWord {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn as_markdown_and_toml_string(&self) -> String {
        let header = toml::to_string(&self.word_header).expect("Can't serialize word header to TOML.");
        format!("``` toml\n{}\n```\n\n{}", &header.trim(), &self.definition_md.trim())
    }

    pub fn from_markdown(s: &str) -> DictWord {
        let a = s.replace("``` toml", "");
        let parts: Vec<&str> = a.split("```").collect();

        let word_header: DictWordHeader = toml::from_str(parts.get(0).unwrap()).unwrap();

        DictWord {
            word_header,
            definition_md: parts.get(1).unwrap().to_string(),
        }
    }
}

impl Default for DictWord {
    fn default() -> Self {
        DictWord {
            word_header: DictWordHeader::default(),
            definition_md: "definition".to_string(),
        }
    }
}

impl Default for DictWordHeader {
    fn default() -> Self {
        DictWordHeader {
            dict_label: "ABCD".to_string(),
            word: "word".to_string(),
            summary: "summary".to_string(),
            grammar: "m.".to_string(),
            inflections: Vec::new(),
        }
    }
}

