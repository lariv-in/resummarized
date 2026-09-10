//! Publisher plugin — subscriber list for newsletter / publication mailing.

pub mod apps;
pub mod create_modals;
pub mod crumbs;
pub mod entities;
pub mod forms;
pub mod handlers;
pub mod keys;
pub mod migrations;
pub mod routes;
pub mod scope;
pub mod state;
pub mod templates;

use frunk::{HCons, hlist::HList};
use lariv_rs::{
    app::App,
    capability::CapStore,
    db::{DbCap, DbTag},
    define_passthrough_cap, define_plugin_install,
    hooks::AttachState,
    traits::{
        add::{AddCapability, CapTagAbsent},
        get::GetByCapTag,
    },
};

use state::PublisherState;

/// Plugin identity tag.
pub struct PublisherTag;

define_passthrough_cap!(PublisherStateCap, PublisherTag, PublisherState);

define_plugin_install! {
    plugin: PublisherTag;
    /// Register Publisher migrations, routes, templates, and dashboard tile.
    steps: [
        apps(apps::Hook),
        migrations(migrations::Hook),
        templates(templates::Hook),
        slots(templates::SlotsHook),
        http(routes::Hook),
        state(StateHook),
    ]
}

/// Attach DB-backed plugin state after the database is connected.
#[derive(Clone, Copy, Default)]
pub struct StateHook;

impl<L, DbIdx, TagProof> AttachState<L, (DbIdx, TagProof)> for StateHook
where
    L: GetByCapTag<DbTag, DbIdx, Value = DbCap>,
    L: HList + CapTagAbsent<PublisherTag, TagProof>,
{
    type Output = HCons<PublisherStateCap, L>;

    fn attach_state(app: App<L>) -> App<Self::Output> {
        let conn = app.get_capability::<DbTag, DbIdx>().items.conn.clone();
        app.add_capability(CapStore::with_items(PublisherState::new(conn)))
    }
}
