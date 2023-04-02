use std::fmt::Debug;
use std::str::FromStr;

use reqwest::Client;
use thiserror::Error;

use crate::steam::{GroupId64, GroupId8, GroupIdentifier, GroupUrl, ToLink};

#[derive(Error, Debug)]
pub enum Error {
    #[error("couldn't find field `{0}`")]
    FieldNotFound(&'static str),
    #[error("field `{0}` is empty")]
    FieldEmpty(&'static str),
    #[error("field `{0}` has invalid content")]
    FieldInvalid(&'static str),
    #[error(transparent)]
    Parse(#[from] roxmltree::Error),
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
}

pub struct GroupData {
    pub name: String,
    pub id_8: GroupId8,
    pub id_64: GroupId64,
    pub url: GroupUrl,
}

impl GroupData {
    fn parse_xml(xml: &str) -> Result<GroupData, Error> {
        macro_rules! find_tag {
            ($node:expr, $tag:ident) => {
                $node
                    .children()
                    .find(|n| n.has_tag_name(stringify!($tag)))
                    .ok_or(Error::FieldNotFound(stringify!($tag)))?
            };
        }
        macro_rules! find_text {
            ($node:expr, $tag:ident) => {
                find_tag!($node, $tag)
                    .text()
                    .ok_or(Error::FieldEmpty(stringify!($tag)))?
            };
        }

        let doc = roxmltree::Document::parse(xml)?;
        let member_list_node = find_tag!(doc.root(), memberList);

        let group_id_64_str = find_text!(member_list_node, groupID64);
        let group_id_64: u64 = group_id_64_str
            .parse()
            .map_err(|_| Error::FieldInvalid("groupID64"))?;

        let group_details_node = find_tag!(member_list_node, groupDetails);
        let group_url = find_text!(group_details_node, groupURL);
        let group_name = find_text!(group_details_node, groupName);

        let group_id_64 =
            GroupId64::try_from(group_id_64).map_err(|_| Error::FieldInvalid("groupID64"))?;

        Ok(GroupData {
            name: group_name.to_string(),
            id_64: group_id_64,
            id_8: GroupId8::from(group_id_64),
            url: GroupUrl::from_str(group_url).map_err(|_| Error::FieldInvalid("groupURL"))?,
        })
    }
    pub async fn fetch(client: &Client, identifier: GroupIdentifier) -> Result<GroupData, Error> {
        let req = client.get(identifier.to_xml_link());
        let xml = req.send().await?.text().await?;
        GroupData::parse_xml(&xml)
    }
}

impl Debug for GroupData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GroupData")
            .field("ID64", &self.id_64.0)
            .field("ID8", &self.id_8.0)
            .field("Name", &self.name)
            .field("URL", &self.url.to_link())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use crate::group_data::GroupData;

    #[test]
    fn parse() {
        pub const TEST_XML_VALVE: &str = include_str!("test_xml_valve.xml");
        pub const TEST_XML_QWERTY: &str = include_str!("test_xml_qwerty.xml");

        macro_rules! check_xml {
            ($xml:ident, $gid:literal, $name:literal, $url:literal) => {
                let xml = GroupData::parse_xml($xml).unwrap();
                assert_eq!(xml.id_64.0, $gid);
                assert_eq!(xml.name, $name);
                assert_eq!(xml.url.0, $url);
            };
        }

        check_xml!(TEST_XML_VALVE, 103582791429521412, "Valve", "Valve");
        check_xml!(
            TEST_XML_QWERTY,
            103582791430191435,
            "Team - [S]-Gamers.cL",
            "qwerty"
        );
    }
}
