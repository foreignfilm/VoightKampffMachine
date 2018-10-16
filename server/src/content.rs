use std::fs::File;
use std::io::prelude::*;
use std::fs;
use serde_json;

pub enum Root {
    SuspectNotes, Penalties, Packets
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Packet {
    name: String,
    pub title: String
}

pub enum PacketContent {
    Info, PatientRobotTypes
}

mod path {

    const ROOT: &str = "../content";

    impl super::Root {
        fn to_string(self) -> String {
            match self {
                super::Root::SuspectNotes => "suspect_notes.json".to_string(),
                super::Root::Penalties => "penalties.json".to_string(),
                super::Root::Packets => "packets/".to_string()
            }
        }
    }

    impl super::PacketContent {
        fn to_string(self) -> String {
            match self {
                super::PacketContent::Info => "packet_info.json".to_string(),
                super::PacketContent::PatientRobotTypes => "patient_robot_types.json".to_string()
            }
        }
    }

    pub fn build_path(content: super::Root, lang: &str) -> String {
        format!("{}/{}/{}", ROOT, lang, content.to_string())
    }

    pub fn build_packet_content_path(packet_name: String, content: super::PacketContent, lang: &str) -> String {
        format!("{}/{}/{}", build_path(super::Root::Packets, lang), packet_name, content.to_string())
    }

}

#[derive(Serialize, Deserialize, Debug)]
pub struct SuspectNote {
    pub title: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Penalty {
    pub title: String,
}

fn content_json(path: String) -> String {
    let mut file = File::open(path)
        .expect("file not found");

    let mut content_json_string = String::new();
    file.read_to_string(&mut content_json_string)
        .expect("something went wrong reading the file");

    return content_json_string
}

lazy_static! {

    pub static ref suspect_notes: Vec<SuspectNote> = {
        // TODO: localisation support?
        let lang = "en";
        let path = path::build_path(Root::SuspectNotes, lang);

        let json_string = content_json(path);
        serde_json::from_str(&json_string)
            .expect("something went wrong parsing this")
    };

    pub static ref penalties: Vec<Penalty> = {
        // TODO: localisation support?
        let lang = "en";
        let path = path::build_path(Root::Penalties, lang);

        let json_string = content_json(path);
        serde_json::from_str(&json_string)
            .expect("something went wrong parsing this")
    };

    pub static ref packets: Vec<Packet> = {
        // TODO: localisation support?
        let lang = "en";

        let packets_dir = path::build_path(Root::Packets, lang);
        let paths = fs::read_dir(packets_dir).unwrap(); 
        return paths
            .map(|path| {
                let packet_name = path.unwrap().file_name().into_string().unwrap();
                print!("{}", packet_name);
                let packet_info_path = path::build_packet_content_path(packet_name, PacketContent::Info, lang);
                let packet_info_json_string = content_json(packet_info_path);
                serde_json::from_str(&packet_info_json_string)
                    .expect("something went wrong parsing this")
            })
            .collect::<Vec<Packet>>();
    };

}
