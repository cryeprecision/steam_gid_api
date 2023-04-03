use std::str::FromStr;
use std::{fmt::Debug, sync::Arc};

use reqwest::Client;
use thiserror::Error;

use crate::steam::{GroupId64, GroupId8, GroupIdentifier, GroupUrl, ToLink};

const PAGE_SIZE: usize = 1_000;

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
    pub avatar: String,
    pub member_count: usize,
    pub members: Vec<u64>,
}

impl GroupData {
    async fn extract_group_info(xml: Arc<String>) -> Result<GroupData, Error> {
        tokio::task::spawn_blocking(move || -> Result<GroupData, Error> {
            let doc = roxmltree::Document::parse(xml.as_str())?;
            let member_list_node = find_tag!(doc.root(), memberList);

            let group_id_64_str = find_text!(member_list_node, groupID64);
            let group_id_64: u64 = group_id_64_str
                .parse()
                .map_err(|_| Error::FieldInvalid("groupID64"))?;

            let member_count_str = find_text!(member_list_node, memberCount);
            let member_count: usize = member_count_str
                .parse()
                .map_err(|_| Error::FieldInvalid("memberCount"))?;

            let group_details_node = find_tag!(member_list_node, groupDetails);

            let group_url = find_text!(group_details_node, groupURL);
            let group_name = find_text!(group_details_node, groupName);
            let avatar = find_text!(group_details_node, avatarFull);

            let group_id_64 =
                GroupId64::try_from(group_id_64).map_err(|_| Error::FieldInvalid("groupID64"))?;

            Ok(GroupData {
                name: group_name.to_string(),
                id_64: group_id_64,
                id_8: GroupId8::from(group_id_64),
                url: GroupUrl::from_str(group_url).map_err(|_| Error::FieldInvalid("groupURL"))?,
                avatar: avatar.to_string(),
                member_count,
                members: Vec::new(),
            })
        })
        .await
        .unwrap()
    }

    async fn extract_members(xml: Arc<String>) -> Result<Vec<u64>, Error> {
        tokio::task::spawn_blocking(move || -> Result<Vec<u64>, Error> {
            let doc = roxmltree::Document::parse(xml.as_str())?;
            let member_list_node = find_tag!(doc.root(), memberList);
            let members_node = find_tag!(member_list_node, members);

            let mut members = Vec::with_capacity(PAGE_SIZE);
            for member_node in members_node
                .children()
                .filter(|n| n.has_tag_name("steamID64"))
            {
                let member_id_str = member_node.text().ok_or(Error::FieldEmpty("steamID64"))?;
                let member_id: u64 = member_id_str
                    .parse()
                    .map_err(|_| Error::FieldInvalid("steamID64"))?;
                members.push(member_id);
            }

            members.sort_unstable();
            members.dedup();

            Ok(members)
        })
        .await
        .unwrap()
    }

    pub async fn fetch(client: &Client, identifier: GroupIdentifier) -> Result<GroupData, Error> {
        let req = client.get(identifier.to_xml_link());
        let xml = Arc::new(req.send().await?.text().await?);

        let members = Self::extract_members(Arc::clone(&xml)).await?;
        let mut group_info = Self::extract_group_info(Arc::clone(&xml)).await?;
        group_info.members = members;

        Ok(group_info)
    }
}

impl Debug for GroupData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GroupData")
            .field("ID64", &self.id_64.0)
            .field("ID8", &self.id_8.0)
            .field("Name", &self.name)
            .field("URL", &self.url.to_link())
            .field("Member Count", &self.member_count)
            .field("Members", &self.members)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use reqwest::Client;

    use crate::{group_data::GroupData, GroupId64, GroupIdentifier};

    #[tokio::test(flavor = "current_thread")]
    async fn fetch_members() {
        let client = Client::new();

        let gd = GroupData::fetch(
            &client,
            GroupIdentifier::Id64(GroupId64(103582791440160998)),
        )
        .await
        .unwrap();

        println!("{gd:#?}");
    }
}
