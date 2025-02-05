use warp::reject::Reject;
use mongodb::error::Error as MongoError;
use bson::oid::Error as BsonError;

#[derive(Debug)]
pub struct MongoRejection(pub MongoError);

#[derive(Debug)]
pub struct BsonRejection(pub BsonError);

impl Reject for MongoRejection {}
impl Reject for BsonRejection {}
