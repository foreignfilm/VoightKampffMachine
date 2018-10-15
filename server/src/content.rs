use std::fs::File;
use std::io::prelude::*;
use serde_json;

pub enum ContentCategory {
    SuspectBackgrounds, Penalties
}

mod path {
    
    const ROOT: &str = "../content";

    impl super::ContentCategory {
        fn to_string(self) -> String {
            match self {
                super::ContentCategory::SuspectBackgrounds => "suspect_backgrounds.json".to_string(),
                super::ContentCategory::Penalties => "penalties.json".to_string()
            }
        }
    }

    pub fn resolve(content: super::ContentCategory, lang: &str) -> String {
        format!("{}/{}/{}", ROOT, lang, content.to_string())
    }

}

#[derive(Serialize, Deserialize, Debug)]
pub struct SuspectBackground {
    pub title: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Penalty {
    pub title: String,
}

fn content_json(content: ContentCategory) -> String {
    // TODO: localisation support?
    let lang = "en";

    let path = path::resolve(content, lang);

    let mut file = File::open(path)
        .expect("suspect backgrounds content file not found");

    let mut content_json_string = String::new();
    file.read_to_string(&mut content_json_string)
        .expect("something went wrong reading the file");

    return content_json_string
}

lazy_static! {

    pub static ref suspect_backgrounds: Vec<SuspectBackground> = {
        let content = content_json(ContentCategory::SuspectBackgrounds);
        serde_json::from_str(&content)
            .expect("something went wrong parsing this")
    };

    pub static ref penalties: Vec<Penalty> = {
        let content = content_json(ContentCategory::Penalties);
        serde_json::from_str(&content)
            .expect("something went wrong parsing this")
    };

}
