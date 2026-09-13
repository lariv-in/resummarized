//! NSE RSS ingest plugin — official feeds from https://www.nseindia.com/static/rss-feed.

pub mod apps;
pub mod brsr;
pub mod config;
pub mod description;
pub mod entities;
pub mod extras;
pub mod feeds;
pub mod fetch;
pub mod fr;
pub mod handlers;
pub mod ic;
pub mod iff;
pub mod it;
pub mod keys;
pub mod migrations;
pub mod routes;
pub mod rpt;
pub mod rss;
pub mod rune_env;
pub mod scr;
pub mod shp;
pub mod sod;
pub mod state;
pub mod stock_market;
pub mod templates;
pub mod uhp;
pub mod voting;
pub mod workers;
pub mod xbrl;

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

use config::{NseConfig, NseConfigTag};
use state::NseState;

/// Plugin identity tag.
pub struct NseTag;

define_passthrough_cap!(NseStateCap, NseTag, NseState);

define_plugin_install! {
    plugin: NseTag;
    /// Register NSE RSS migrations, routes, templates, dashboard tile, and poller.
    steps: [
        cap_hook(crate::stock_markets::StockMarketTag, crate::stock_markets::StockMarketsCap, stock_market::Hook),
        apps(apps::Hook),
        rune_env(rune_env::Hook),
        migrations(migrations::Hook),
        templates(templates::Hook),
        slots(templates::SlotsHook),
        config(NseConfigTag, NseConfig),
        http(routes::Hook),
        state(StateHook),
        serve_startup(workers::ServeStartupHook),
    ]
}

#[derive(Clone, Copy, Default)]
pub struct StateHook;

impl<L, DbIdx, CfgIdx, Configs, NseCfgIdx, TagProof>
    AttachState<L, (DbIdx, CfgIdx, Configs, NseCfgIdx, TagProof)> for StateHook
where
    L: GetByCapTag<DbTag, DbIdx, Value = DbCap>,
    L: GetByCapTag<ConfigTag, CfgIdx, Value = ConfigCap<HNil, Configs>>,
    Configs: GetByTag<NseConfigTag, NseCfgIdx, Value = NseConfig>,
    L: HList + CapTagAbsent<NseTag, TagProof>,
{
    type Output = HCons<NseStateCap, L>;

    fn attach_state(app: App<L>) -> App<Self::Output> {
        let conn = app.get_capability::<DbTag, DbIdx>().items.conn.clone();
        let config = <Configs as GetByTag<NseConfigTag, NseCfgIdx>>::get_by_tag(
            &app.get_capability::<ConfigTag, CfgIdx>().items,
        )
        .clone();
        let client = fetch::build_client().expect("NSE HTTP client");
        app.add_capability(CapStore::with_items(NseState::new(conn, config, client)))
    }
}
