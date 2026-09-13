//! JPX (Japan Exchange Group) RSS ingest plugin — official English feeds from
//! https://www.jpx.co.jp/english/rss/index.html.

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
pub mod stock_market;
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

use config::{JpxConfig, JpxConfigTag};
use state::JpxState;

/// Plugin identity tag.
pub struct JpxTag;

define_passthrough_cap!(JpxStateCap, JpxTag, JpxState);

define_plugin_install! {
    plugin: JpxTag;
    /// Register JPX RSS migrations, routes, templates, dashboard tile, and poller.
    steps: [
        cap_hook(crate::stock_markets::StockMarketTag, crate::stock_markets::StockMarketsCap, stock_market::Hook),
        apps(apps::Hook),
        rune_env(rune_env::Hook),
        migrations(migrations::Hook),
        templates(templates::Hook),
        slots(templates::SlotsHook),
        config(JpxConfigTag, JpxConfig),
        http(routes::Hook),
        state(StateHook),
        serve_startup(workers::ServeStartupHook),
    ]
}

#[derive(Clone, Copy, Default)]
pub struct StateHook;

impl<L, DbIdx, CfgIdx, Configs, JpxCfgIdx, TagProof>
    AttachState<L, (DbIdx, CfgIdx, Configs, JpxCfgIdx, TagProof)> for StateHook
where
    L: GetByCapTag<DbTag, DbIdx, Value = DbCap>,
    L: GetByCapTag<ConfigTag, CfgIdx, Value = ConfigCap<HNil, Configs>>,
    Configs: GetByTag<JpxConfigTag, JpxCfgIdx, Value = JpxConfig>,
    L: HList + CapTagAbsent<JpxTag, TagProof>,
{
    type Output = HCons<JpxStateCap, L>;

    fn attach_state(app: App<L>) -> App<Self::Output> {
        let conn = app.get_capability::<DbTag, DbIdx>().items.conn.clone();
        let config = <Configs as GetByTag<JpxConfigTag, JpxCfgIdx>>::get_by_tag(
            &app.get_capability::<ConfigTag, CfgIdx>().items,
        )
        .clone();
        let client = fetch::build_client().expect("JPX HTTP client");
        app.add_capability(CapStore::with_items(JpxState::new(conn, config, client)))
    }
}
