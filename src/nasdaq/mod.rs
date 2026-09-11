//! Nasdaq, Inc. IR RSS ingest plugin — official feeds from
//! https://ir.nasdaq.com/tools/rss-feeds.

pub mod apps;
pub mod atom;
pub mod config;
pub mod entities;
pub mod feeds;
pub mod fetch;
pub mod handlers;
pub mod keys;
pub mod migrations;
pub mod query;
pub mod routes;
pub mod rss;
pub mod rune_env;
pub mod state;
pub mod templates;
pub mod workers;

use frunk::{HCons, HNil, hlist::HList};
use lariv_rs::{
    app::App,
    capability::CapStore,
    config::{ConfigCap, ConfigTag},
    db::{DbCap, DbTag},
    define_passthrough_cap, define_plugin_install,
    hooks::AttachState,
    traits::{
        add::{AddCapability, CapTagAbsent},
        get::{GetByCapTag, GetByTag},
    },
};

use config::{NasdaqConfig, NasdaqConfigTag};
use state::NasdaqState;

/// Plugin identity tag.
pub struct NasdaqTag;

define_passthrough_cap!(NasdaqStateCap, NasdaqTag, NasdaqState);

define_plugin_install! {
    plugin: NasdaqTag;
    /// Register Nasdaq RSS migrations, routes, templates, dashboard tile, and poller.
    steps: [
        apps(apps::Hook),
        rune_env(rune_env::Hook),
        migrations(migrations::Hook),
        templates(templates::Hook),
        slots(templates::SlotsHook),
        config(NasdaqConfigTag, NasdaqConfig),
        http(routes::Hook),
        state(StateHook),
        serve_startup(workers::ServeStartupHook),
    ]
}

#[derive(Clone, Copy, Default)]
pub struct StateHook;

impl<L, DbIdx, CfgIdx, Configs, NasdaqCfgIdx, TagProof>
    AttachState<L, (DbIdx, CfgIdx, Configs, NasdaqCfgIdx, TagProof)> for StateHook
where
    L: GetByCapTag<DbTag, DbIdx, Value = DbCap>,
    L: GetByCapTag<ConfigTag, CfgIdx, Value = ConfigCap<HNil, Configs>>,
    Configs: GetByTag<NasdaqConfigTag, NasdaqCfgIdx, Value = NasdaqConfig>,
    L: HList + CapTagAbsent<NasdaqTag, TagProof>,
{
    type Output = HCons<NasdaqStateCap, L>;

    fn attach_state(app: App<L>) -> App<Self::Output> {
        let conn = app.get_capability::<DbTag, DbIdx>().items.conn.clone();
        let config = <Configs as GetByTag<NasdaqConfigTag, NasdaqCfgIdx>>::get_by_tag(
            &app.get_capability::<ConfigTag, CfgIdx>().items,
        )
        .clone();
        let client = fetch::build_client().expect("Nasdaq HTTP client");
        app.add_capability(CapStore::with_items(NasdaqState::new(conn, config, client)))
    }
}
