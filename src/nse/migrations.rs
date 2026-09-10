use sea_orm_migration::prelude::*;

use super::NseTag;

mod m00001_create_nse;
mod m00002_item_as_on_date;
mod m00003_item_description_fields;
mod m00004_typed_dates;
mod m00005_brsr_xbrl;
mod m00006_voting_xbrl;
mod m00007_uhp_xbrl;
mod m00008_sod_xbrl;
mod m00009_rename_vote_columns;
mod m00010_sod_object_fields;
mod m00011_sod_objects_json;
mod m00012_shp_xbrl;
mod m00013_scr_xbrl;
mod m00014_five_xbrl;
mod m00015_pg_trgm;
mod m00016_split_feed_tables;

#[derive(Clone, Copy, Default)]
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m00001_create_nse::Migration),
            Box::new(m00002_item_as_on_date::Migration),
            Box::new(m00003_item_description_fields::Migration),
            Box::new(m00004_typed_dates::Migration),
            Box::new(m00005_brsr_xbrl::Migration),
            Box::new(m00006_voting_xbrl::Migration),
            Box::new(m00007_uhp_xbrl::Migration),
            Box::new(m00008_sod_xbrl::Migration),
            Box::new(m00009_rename_vote_columns::Migration),
            Box::new(m00010_sod_object_fields::Migration),
            Box::new(m00011_sod_objects_json::Migration),
            Box::new(m00012_shp_xbrl::Migration),
            Box::new(m00013_scr_xbrl::Migration),
            Box::new(m00014_five_xbrl::Migration),
            Box::new(m00015_pg_trgm::Migration),
            Box::new(m00016_split_feed_tables::Migration),
        ]
    }
}

lariv_rs::define_register_migrations! {
    plugin: NseTag;
    migrator: Migrator;
}
