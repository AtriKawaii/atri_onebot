use atri_plugin::contact::friend::Friend;
use atri_plugin::contact::group::Group;
use atri_plugin::contact::member::NamedMember;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct UserInfo {
    pub user_id: String,
    pub user_name: String,
    pub user_displayname: String,
    pub user_remark: String,
}

impl From<Friend> for UserInfo {
    fn from(f: Friend) -> Self {
        Self {
            user_id: f.id().to_string(),
            user_name: f.nickname().to_string(),
            user_displayname: "".to_string(),
            user_remark: f.nickname().to_string(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct GroupInfo {
    pub group_id: String,
    pub group_name: String,
}

impl From<Group> for GroupInfo {
    fn from(g: Group) -> Self {
        Self {
            group_id: g.id().to_string(),
            group_name: g.name().to_string(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct GroupMemberInfo {
    pub user_id: String,
    pub user_name: String,
    pub user_displayname: String,
}

impl From<NamedMember> for GroupMemberInfo {
    fn from(named: NamedMember) -> Self {
        Self {
            user_id: named.id().to_string(),
            user_name: named.nickname().to_string(),
            user_displayname: named.card_name().to_string(),
        }
    }
}
