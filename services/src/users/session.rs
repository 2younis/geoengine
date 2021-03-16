use serde::{Deserialize, Serialize};

use crate::projects::project::{ProjectId, STRectangle};
use crate::users::user::UserId;
use crate::util::Identifier;
use chrono::{DateTime, Utc};
use geoengine_datatypes::identifier;

identifier!(SessionId);

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct UserInfo {
    pub id: UserId,
    pub email: Option<String>,
    pub real_name: Option<String>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Session {
    pub id: SessionId,
    pub user: UserInfo,
    pub created: DateTime<Utc>,
    pub valid_until: DateTime<Utc>,
    pub project: Option<ProjectId>,
    pub view: Option<STRectangle>,
}

impl Session {
    pub fn mock() -> Self {
        Self {
            id: SessionId::new(),
            user: UserInfo {
                id: UserId::new(),
                email: None,
                real_name: None,
            },
            created: chrono::Utc::now(),
            valid_until: chrono::Utc::now(),
            project: None,
            view: None,
        }
    }
}
