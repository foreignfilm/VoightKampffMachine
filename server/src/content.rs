use std::fs;
extern crate serde;

mod path {

    use content::serde::de::DeserializeOwned;

    const ROOT: &str = "../content";

    pub enum GeneralContent {
        SuspectNotes, Penalties, Packets,
        HumanRoleCardTemplate, ViolentRobotRoleCardTemplate, PatientRobotRoleCardTemplate,
        PrimaryPromptCardTemplate, SecondaryPromptCardTemplate
    }

    impl GeneralContent {
        fn to_string(self) -> String {
            match self {
                GeneralContent::SuspectNotes => "suspect_notes.json".to_string(),
                GeneralContent::Penalties => "penalties.json".to_string(),
                GeneralContent::Packets => "packets/".to_string(),
                GeneralContent::HumanRoleCardTemplate => "card_templates/human.json".to_string(),
                GeneralContent::ViolentRobotRoleCardTemplate => "card_templates/violent_robot.json".to_string(),
                GeneralContent::PatientRobotRoleCardTemplate => "card_templates/patient_robot.json".to_string(),
                GeneralContent::PrimaryPromptCardTemplate => "card_templates/primary_prompt.json".to_string(),
                GeneralContent::SecondaryPromptCardTemplate => "card_templates/secondary_prompt.json".to_string()
            }
        }
    }

    pub enum PacketContent {
        Info, PatientRobots, ViolentRobots, PrimaryPrompts, SecondaryPrompts
    }

    impl PacketContent {
        fn to_string(self) -> String {
            match self {
                PacketContent::Info => "packet_info.json".to_string(),
                PacketContent::PatientRobots => "patient_robots.json".to_string(),
                PacketContent::ViolentRobots => "violent_robots.json".to_string(),
                PacketContent::PrimaryPrompts => "primary_prompts.json".to_string(),
                PacketContent::SecondaryPrompts => "secondary_prompts.json".to_string()
            }
        }
    }

    pub fn build_path(content: GeneralContent, lang: &str) -> String {
        format!("{}/{}/{}", ROOT, lang, content.to_string())
    }

    pub fn build_packet_content_path(packet_name: String, content: PacketContent, lang: &str) -> String {
        format!("{}/{}/{}", build_path(GeneralContent::Packets, lang), packet_name, content.to_string())
    }

    pub fn parse_from_file<T>(path: String) -> T where T: DeserializeOwned {
        use std::fs::File;
        use std::io::Read;
        use serde_json;

        let mut file = File::open(path)
            .expect("file not found");

        let mut content_json_string = String::new();
        file.read_to_string(&mut content_json_string)
            .expect("something went wrong reading the file");

        return serde_json::from_str(&content_json_string)
            .expect("something went wrong parsing this")
    }

}

// Content Models

#[derive(Serialize, Deserialize)]
pub struct Packet {
    name: String,
    pub title: String
}

#[derive(Serialize, Deserialize)]
pub struct SuspectNote {
    pub title: String
}

#[derive(Serialize, Deserialize)]
pub struct Penalty {
    pub title: String
}

#[derive(Serialize, Deserialize)]
pub struct HumanRoleCardTemplate {
    pub title: String,
    pub center_text: String,
    pub bottom_text: String,
    pub footnote: String
}

#[derive(Serialize, Deserialize)]
pub struct ViolentRobot {
    pub obsession: String,
    pub objective1: String,
    pub objective2: String,
}

#[derive(Serialize, Deserialize)]
pub struct PatientRobot {
    pub vulnerability: String,
    pub vulnerability_description: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ViolentRobotRoleCardTemplate {
    pub title: String,
    pub subtitle: String,
    pub extra_objective: String,
    pub footnote: String
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PatientRobotRoleCardTemplate {
    pub title: String,
    pub subtitle: String,
    pub footnote: String
}

#[derive(Serialize, Deserialize)]
pub struct Prompt {
    pub prompt: String,
    pub example_question1: String,
    pub example_question2: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PromptCardTemplate {
    #[serde(default)]
    pub is_primary: bool,
    #[serde(default)]
    pub is_secondary: bool,

    pub title: String,
    pub pre_prompt: String,
    pub post_prompt: String,
    pub questions_label: String,

    #[serde(default)]
    pub pre_prompt_extra: String
}

impl Default for PromptCardTemplate {
    fn default() -> Self {
        use content;
        // FIXME: could this be a reference?
        // FIXME: !!! this always provides primary prompt templates !!!
        content::primary_prompt_card_template.clone()
    }
}

// Content retrieval methods

pub fn violent_robots(packet_name: String) -> Vec<(ViolentRobot, ViolentRobotRoleCardTemplate)> { 
    // TODO: localisation support?
    let lang = "en";
    path::parse_from_file(
        path::build_packet_content_path(packet_name, path::PacketContent::ViolentRobots, lang)
    )
}

pub fn patient_robots(packet_name: String) -> Vec<PatientRobot> {
    // TODO: localisation support?
    let lang = "en";
    path::parse_from_file(
        path::build_packet_content_path(packet_name, path::PacketContent::PatientRobots, lang)
    )
}

pub fn primary_prompts(packet_name: String) -> Vec<Prompt> { 
    // TODO: localisation support?
    let lang = "en";
    path::parse_from_file(
        path::build_packet_content_path(packet_name, path::PacketContent::PrimaryPrompts, lang)
    )
}

pub fn secondary_prompts(packet_name: String) -> Vec<Prompt> {
    // TODO: localisation support?
    let lang = "en";
    path::parse_from_file(
        path::build_packet_content_path(packet_name, path::PacketContent::SecondaryPrompts, lang)
    )
}

lazy_static! {

    pub static ref suspect_notes: Vec<SuspectNote> = {
        // TODO: localisation support?
        let lang = "en";
        path::parse_from_file(
            path::build_path(path::GeneralContent::SuspectNotes, lang)
        )
    };

    pub static ref penalties: Vec<Penalty> = {
        // TODO: localisation support?
        let lang = "en";
        path::parse_from_file(
            path::build_path(path::GeneralContent::Penalties, lang)
        )
    };

    pub static ref packets: Vec<Packet> = {
        // TODO: localisation support?
        let lang = "en";

        let packets_dir = path::build_path(path::GeneralContent::Packets, lang);
        let paths = fs::read_dir(packets_dir).unwrap(); 
        return paths
            .map(|path| {
                let packet_name = path.unwrap().file_name().into_string().unwrap();
                path::parse_from_file(
                    path::build_packet_content_path(packet_name, path::PacketContent::Info, lang)
                )
            })
            .collect::<Vec<Packet>>();
    };

    pub static ref human_role_card_template: HumanRoleCardTemplate = {
        // TODO: localisation support?
        let lang = "en";
        path::parse_from_file(
            path::build_path(path::GeneralContent::HumanRoleCardTemplate, lang)
        )
    };

    pub static ref violent_robot_role_card_template: ViolentRobotRoleCardTemplate = {
        // TODO: localisation support?
        let lang = "en";
        path::parse_from_file(
            path::build_path(path::GeneralContent::ViolentRobotRoleCardTemplate, lang)
        )
    };

    pub static ref patient_robot_role_card_template: PatientRobotRoleCardTemplate = {
        // TODO: localisation support?
        let lang = "en";
        path::parse_from_file(
            path::build_path(path::GeneralContent::PatientRobotRoleCardTemplate, lang)
        )
    };

    pub static ref primary_prompt_card_template: PromptCardTemplate = {
        // TODO: localisation support?
        let lang = "en";
        path::parse_from_file(
            path::build_path(path::GeneralContent::PrimaryPromptCardTemplate, lang)
        )
    };

    pub static ref secondary_prompt_card_template: PromptCardTemplate = {
        // TODO: localisation support?
        let lang = "en";
        path::parse_from_file(
            path::build_path(path::GeneralContent::SecondaryPromptCardTemplate, lang)
        )
    };

}
