use std::fs::File;
use std::io::prelude::*;
use serde_json;

mod path {
    const ROOT: &str = "../content";

    pub enum ContentCategory {
        SuspectBackgrounds
    }

    impl ContentCategory {
        fn to_string(self) -> String {
            match self {
                ContentCategory::SuspectBackgrounds => "suspect_backgrounds.json".to_string()
            }
        }
    }

    pub fn to(content: ContentCategory, lang: &str) -> String {
        format!("{}/{}/{}", ROOT, lang, content.to_string())
    }

}

#[derive(Serialize, Deserialize, Debug)]
pub struct SuspectBackground {
    pub name: String,
}

lazy_static! {
    pub static ref suspect_backgrounds: Vec<SuspectBackground> = {
        let path = path::to(path::ContentCategory::SuspectBackgrounds, "en");

        let mut file = File::open(path)
            .expect("suspect_backgrounds content file not found");

        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .expect("something went wrong reading the file");

        return serde_json::from_str(&contents)
            .expect("something went wrong parsing this")
    };
}
