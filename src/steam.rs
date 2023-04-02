use std::str::FromStr;

use lazy_regex::regex_is_match;
use log::info;
use serde::Serialize;
use thiserror::Error;

pub const GID_OFFSET: u64 = 0x0170_0000_0000_0000;

#[derive(Debug, Error)]
pub enum UrlError {
    #[error("URL has invalid suffix")]
    InvalidSuffix,
}

#[derive(Debug, Error)]
pub enum IdError {
    #[error("ID is out of range")]
    OutOfRange,
}

#[derive(Debug, Error)]
pub enum IdentifierError {
    #[error(transparent)]
    Url(#[from] UrlError),
    #[error(transparent)]
    Id(#[from] IdError),
    #[error("identifier was not recognized")]
    Unrecognized,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct GroupId64(pub u64);

#[derive(Debug, Clone, Copy, Serialize)]
pub struct GroupId8(pub u64);

#[derive(Debug, Serialize)]
pub struct GroupUrl(pub String);

#[derive(Debug)]
pub enum GroupIdentifier {
    Id64(GroupId64),
    Id8(GroupId8),
    Url(GroupUrl),
}

pub trait ToLink {
    fn to_link(&self) -> String;
    fn to_xml_link(&self) -> String;
}

impl ToLink for GroupId64 {
    fn to_link(&self) -> String {
        format!("https://steamcommunity.com/gid/{}", self.0)
    }
    fn to_xml_link(&self) -> String {
        format!(
            "https://steamcommunity.com/gid/{}/memberslistxml/?xml=1",
            self.0
        )
    }
}
impl ToLink for GroupId8 {
    fn to_link(&self) -> String {
        GroupId64::from(*self).to_link()
    }
    fn to_xml_link(&self) -> String {
        GroupId64::from(*self).to_xml_link()
    }
}
impl ToLink for GroupUrl {
    fn to_link(&self) -> String {
        format!("https://steamcommunity.com/groups/{}", self.0)
    }
    fn to_xml_link(&self) -> String {
        format!(
            "https://steamcommunity.com/groups/{}/memberslistxml/?xml=1",
            self.0
        )
    }
}
impl ToLink for GroupIdentifier {
    fn to_link(&self) -> String {
        match self {
            GroupIdentifier::Id64(id_64) => id_64.to_link(),
            GroupIdentifier::Id8(id_8) => id_8.to_link(),
            GroupIdentifier::Url(name) => name.to_link(),
        }
    }
    fn to_xml_link(&self) -> String {
        match self {
            GroupIdentifier::Id64(id_64) => id_64.to_xml_link(),
            GroupIdentifier::Id8(id_8) => id_8.to_xml_link(),
            GroupIdentifier::Url(name) => name.to_xml_link(),
        }
    }
}

impl FromStr for GroupIdentifier {
    type Err = IdentifierError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(id) = s.parse::<u64>() {
            if id >= GID_OFFSET {
                info!("recognized `{s}` as GroupId64");
                Ok(GroupIdentifier::Id64(GroupId64(id)))
            } else {
                info!("recognized `{s}` as GroupId8");
                Ok(GroupIdentifier::Id8(GroupId8(id)))
            }
        } else if s.starts_with("https://steamcommunity.com/groups/")
            || s.starts_with("http://steamcommunity.com/groups/")
            || s.starts_with("steamcommunity.com/groups/")
        {
            info!("recognized `{s}` as URL");
            let (_, group_name) = s.split_once("steamcommunity.com/groups/").unwrap();
            Ok(GroupIdentifier::Url(GroupUrl::from_str(group_name)?))
        } else if let Ok(group_name) = GroupUrl::from_str(s) {
            info!("recognized `{s}` as URL suffix");
            Ok(GroupIdentifier::Url(group_name))
        } else {
            Err(IdentifierError::Unrecognized)
        }
    }
}

impl FromStr for GroupUrl {
    type Err = UrlError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if regex_is_match!(r"^[\w\d\-_]+$", s) {
            Ok(GroupUrl(s.to_string()))
        } else {
            Err(UrlError::InvalidSuffix)
        }
    }
}

impl From<GroupId8> for GroupId64 {
    fn from(id: GroupId8) -> Self {
        GroupId64(id.0.checked_add(GID_OFFSET).unwrap())
    }
}

impl From<GroupId64> for GroupId8 {
    fn from(id: GroupId64) -> Self {
        GroupId8(id.0.checked_sub(GID_OFFSET).unwrap())
    }
}

impl TryFrom<u64> for GroupId64 {
    type Error = IdError;
    fn try_from(id: u64) -> Result<Self, Self::Error> {
        if id < GID_OFFSET {
            Err(IdError::OutOfRange)
        } else {
            Ok(GroupId64(id))
        }
    }
}

impl TryFrom<u64> for GroupId8 {
    type Error = IdError;
    fn try_from(id: u64) -> Result<Self, Self::Error> {
        if id > GID_OFFSET {
            Err(IdError::OutOfRange)
        } else {
            Ok(GroupId8(id))
        }
    }
}
