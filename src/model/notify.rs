use bson::doc;
use mongodb::{Cursor, bson::oid::ObjectId};
use mongodb::bson;
use serde::{Serialize, Deserialize};
use tokio::stream::StreamExt;
use uuid::Uuid;

use super::{Error, ExtractProfile, Model, Service, ValidateProfile, error::mongo_error};

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct NotifyProfile {
    pub smtp_address: String,
    pub tls: bool,
    pub username: String,
    pub password: String,
    pub email_address: String,
    pub name: String,
}

impl ExtractProfile<NotifyProfile> for NotifyProfile {
    fn extract_from(service: &Service) -> Option<&Self> {
        match service {
            Service::EmailNotify(profile) => Some(profile),
            _ => None,
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq)]
pub enum NotifyState {
    Pending,
    Sent,
    /// (pub_error, inner_error)
    Error(String, String),
}

#[derive(Serialize, Deserialize)]
pub struct MailData {
    pub to: String,
    pub subject: String,
    pub content_type: String,
    pub body: String,
}

#[derive(Deserialize, Serialize)]
pub struct EmailNotify {
    pub _id: ObjectId,
    pub message_id: String,
    pub status: NotifyState,
    pub sender_profile: ObjectId,
    pub mail: MailData,
}
impl ValidateProfile for NotifyProfile {
}

const COLLECTION_NOTIFY: &str = "notify";

impl Model {
    pub fn new_email_notify(&self, sender_profile: ObjectId, mail: MailData, sender_addr: &str) -> EmailNotify {
        let message_id = format!("{}.{}", Uuid::new_v4().to_hyphenated().to_string(), sender_addr);
        EmailNotify {
            _id: ObjectId::new(),
            message_id,
            status: NotifyState::Pending,
            sender_profile,
            mail,
        }
    }
    pub async fn get_all_notifications(&self) -> Result<Vec<EmailNotify>, Error> {
        let coll = self.db.collection(COLLECTION_NOTIFY);
        let result = coll.find(None, None)
            .await
            .map_err(mongo_error)?;
        let notifications: Vec<EmailNotify> = result
            .filter_map(|doc| doc.ok().and_then(|d| bson::from_document(d).ok()))
            .collect()
            .await;

        Ok(notifications)
    }

    pub async fn get_notification_by_message_id(&self, message_id: &ObjectId) -> Result<EmailNotify, Error> {
        let coll = self.db.collection(COLLECTION_NOTIFY);
        let query = doc! {
            "_id": message_id,
        };
        let doc = coll.find_one(query, None)
            .await
            .map_err(Error::from)?
            .ok_or(Error::NoRecord)?;

        let notify = bson::from_document(doc).unwrap();
        
        Ok(notify)
    }

    pub async fn add_notification(&self, notify: &EmailNotify) -> Result<(), Error> {
        let coll = self.db.collection(COLLECTION_NOTIFY);
        let doc = bson::to_document(notify).map_err(Error::from)?;
        let result = coll.insert_one(doc, None).await.map_err(Error::from)?;
        Ok(())
    }

    pub async fn update_notification(&self, notify: &EmailNotify) -> Result<(), Error> {
        let coll = self.db.collection(COLLECTION_NOTIFY);
        let query = doc! {
            "_id": &notify._id
        };
        let update = doc!{
            "$set": bson::to_bson(notify).unwrap(),
        };
        let result = coll.update_one(query, update, None).await.map_err(Error::from)?;
        if result.matched_count <= 0 {
            Err(Error::NoRecord)
        } else {
            Ok(())
        }
    }
}