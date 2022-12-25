//! `SeaORM` Entity. Generated by sea-orm-codegen 0.10.6

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "Server")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub address: String,
    pub host: String,
    pub port: u16,
    pub version: String,
    pub protocol: i32,
    pub max_players: u64,
    pub online_players: u64,
    pub response: Json,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
