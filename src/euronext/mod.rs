//! Euronext Athens RSS ingest plugin — official feeds from
//! https://athens.euronext.com/en/rss.

pub mod apps;
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

use config::{EuronextConfig, EuronextConfigTag};
use state::EuronextState;

/// Plugin identity tag.
pub struct EuronextTag;

define_passthrough_cap!(EuronextStateCap, EuronextTag, EuronextState);

define_plugin_install! {
    plugin: EuronextTag;
    /// Register Euronext RSS migrations, routes, templates, dashboard tile, and poller.
    steps: [
        apps(apps::Hook),
        rune_env(rune_env::Hook),
        migrations(migrations::Hook),
        templates(templates::Hook),
        slots(templates::SlotsHook),
        config(EuronextConfigTag, EuronextConfig),
        http(routes::Hook),
        state(StateHook),
        serve_startup(workers::ServeStartupHook),
    ]
}

#[derive(Clone, Copy, Default)]
pub struct StateHook;

impl<L, DbIdx, CfgIdx, Configs, EuronextCfgIdx, TagProof>
    AttachState<L, (DbIdx, CfgIdx, Configs, EuronextCfgIdx, TagProof)> for StateHook
where
    L: GetByCapTag<DbTag, DbIdx, Value = DbCap>,
    L: GetByCapTag<ConfigTag, CfgIdx, Value = ConfigCap<HNil, Configs>>,
    Configs: GetByTag<EuronextConfigTag, EuronextCfgIdx, Value = EuronextConfig>,
    L: HList + CapTagAbsent<EuronextTag, TagProof>,
{
    type Output = HCons<EuronextStateCap, L>;

    fn attach_state(app: App<L>) -> App<Self::Output> {
        let conn = app.get_capability::<DbTag, DbIdx>().items.conn.clone();
        let config = <Configs as GetByTag<EuronextConfigTag, EuronextCfgIdx>>::get_by_tag(
            &app.get_capability::<ConfigTag, CfgIdx>().items,
        )
        .clone();
        let client = fetch::build_client().expect("Euronext HTTP client");
        app.add_capability(CapStore::with_items(EuronextState::new(
            conn, config, client,
        )))
    }
}
