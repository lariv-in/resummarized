//! BSE RSS ingest plugin — official feeds from https://beta.bseindia.com/rss-feed.html.

pub mod apps;
pub mod config;
pub mod description;
pub mod entities;
pub mod extras;
pub mod feeds;
pub mod fetch;
pub mod handlers;
pub mod keys;
pub mod migrations;
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

use config::{BseConfig, BseConfigTag};
use state::BseState;

/// Plugin identity tag.
pub struct BseTag;

define_passthrough_cap!(BseStateCap, BseTag, BseState);

define_plugin_install! {
    plugin: BseTag;
    /// Register BSE RSS migrations, routes, templates, dashboard tile, and poller.
    steps: [
        apps(apps::Hook),
        rune_env(rune_env::Hook),
        migrations(migrations::Hook),
        templates(templates::Hook),
        slots(templates::SlotsHook),
        config(BseConfigTag, BseConfig),
        http(routes::Hook),
        state(StateHook),
        serve_startup(workers::ServeStartupHook),
    ]
}

#[derive(Clone, Copy, Default)]
pub struct StateHook;

impl<L, DbIdx, CfgIdx, Configs, BseCfgIdx, TagProof>
    AttachState<L, (DbIdx, CfgIdx, Configs, BseCfgIdx, TagProof)> for StateHook
where
    L: GetByCapTag<DbTag, DbIdx, Value = DbCap>,
    L: GetByCapTag<ConfigTag, CfgIdx, Value = ConfigCap<HNil, Configs>>,
    Configs: GetByTag<BseConfigTag, BseCfgIdx, Value = BseConfig>,
    L: HList + CapTagAbsent<BseTag, TagProof>,
{
    type Output = HCons<BseStateCap, L>;

    fn attach_state(app: App<L>) -> App<Self::Output> {
        let conn = app.get_capability::<DbTag, DbIdx>().items.conn.clone();
        let config = <Configs as GetByTag<BseConfigTag, BseCfgIdx>>::get_by_tag(
            &app.get_capability::<ConfigTag, CfgIdx>().items,
        )
        .clone();
        let client = fetch::build_client().expect("BSE HTTP client");
        app.add_capability(CapStore::with_items(BseState::new(conn, config, client)))
    }
}
