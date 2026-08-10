//! # Property-store spike harness
//!
//! SQL-physical measurements for the graph property-store spike. This binary
//! does **not** run Cypher. It builds layouts with raw SQL and times them so we
//! can choose product direction before any frontend lowering change.
//!
//! Plan: `docs/superpowers/plans/2026-08-10-graph-json-bag-property-store-spike.md`  
//! Results write-up: `graph/test-results/property_store_spike_phase1b.md`
//!
//! ## What problem this measures
//!
//! 1. **Type explosion:** default CREATE GRAPH tends to one SQL table per node /
//!    relationship type. At thousands of types, `sqlite_schema` and DDL cost blow
//!    up. Shared stores put many types in few tables.
//! 2. **Property physical mode:** after sources are shared, properties may be
//!    fixed columns, a JSON bag column, or indexed cells (EAV on B-trees). Bags
//!    are weak for selective equality filters unless you add hot-key indexes.
//!
//! ## Layouts (configs)
//!
//! | Id | Physical layout | What it stands for |
//! |----|-----------------|--------------------|
//! | C0 | One `node_{type}` table per type; props = columns | Today’s type-per-table scale path |
//! | C1 | One `nodes` + `edges`; props = columns | Shared store, dense known props |
//! | C2 | One `nodes` + `edges`; `props TEXT` JSON | Shared store + text property bag |
//! | C2j | Same as C2 with `props JSONB` (custom types on) | Declared JSONB bag |
//! | C3 | One `nodes` + `prop_dict` + `node_props(prop_id)` + index `(prop_id, value)` | Open Cell store (integer keys) |
//! | C3s | Same cells but `prop_key TEXT` + index `(prop_key, value)` | String-key EAV (baseline for dict win) |
//! | C4 | C2 + expr index on `json_extract(props,'$.pN')` | Hybrid: **TEXT** bag + one hot-key expression index |
//! | C4j | C2j + expr index on `json_extract(props,'$.pN')` | Hybrid: **JSONB** bag + one hot-key expression index |
//!
//! C4 / C4j are optional (`PROPERTY_STORE_SPIKE_INCLUDE_C4=1`). One expression
//! index does not scale to an open property namespace; it models declared hot keys.
//! JSONB configs open the DB with `DatabaseOpts::with_custom_types(true)`.
//!
//! ## Workloads (what each metric means)
//!
//! | Metric / phase | SQL intent | Decides |
//! |----------------|------------|---------|
//! | `phase=schema` `schema_ms`, `table_count` | CREATE TABLE/INDEX only | **H1/H2:** must shared stores win schema wall? |
//! | `load_ms` | bulk INSERT of entities (+ edges on shared) | load cost of bag vs cell |
//! | `filter_eq_ms`, `filter_plan_uses_index` | equality on one property = unique value | **H4:** bag vs indexed property path |
//! | `project_one_ms` | read one property from all (or one type for C0) | **H3** point project |
//! | `project_all_ms` | read full property map / all cells | **H3** whole-map materialize |
//! | `set_one_ms` | update one property on entity id 0 | **H6** bag rewrite vs cell update |
//! | `hop_prop_ms` | 1-hop on `edges` then filter neighbor property | graph-shaped join (edge index may dominate) |
//!
//! C0 project/filter times only touch `node_0` (one type). Do **not** compare
//! them to shared full-table scans.
//!
//! `hop_prop_plan_uses_index` means EXPLAIN saw **any** index (often `edges_start`).
//! It is **not** proof of a property-value index. Use `filter_plan_uses_index`
//! for property-indexability.
//!
//! ## Decision outcomes (A / B / C)
//!
//! Numbers from this harness feed the plan’s decision gate:
//!
//! | Outcome | Choose when | Product next step |
//! |---------|-------------|-------------------|
//! | **A** bag-only | Shared stores win schema wall, and bag filters are acceptable (or rare) | Shared stores + bag AM; no Cell |
//! | **B** cells-for-all | Selective property filters dominate; open prop namespace needs indexes on many keys | `PropertyPhysical::Cell` on B-trees |
//! | **C** hybrid | Bag wins map project; filters need indexes only on hot/merge keys | Bag (or columns) + Cell and/or expr index for hot keys |
//!
//! Phase 1b chose **C** (see `property_store_spike_phase1b.md`). This harness
//! still prints raw metrics so later runs can re-check that gate.
//!
//! ## Non-goals of this binary
//!
//! - Cypher parse/bind/lower
//! - Shipping CREATE GRAPH defaults
//! - Porting smoltable / LSM wide-column into core
//!
//! ## Env knobs
//!
//! - `PROPERTY_STORE_SPIKE_TYPES` (default 50)
//! - `PROPERTY_STORE_SPIKE_ENTITIES` (default 20 per type)
//! - `PROPERTY_STORE_SPIKE_PROPS` (default 8)
//! - `PROPERTY_STORE_SPIKE_FILTER_PROP` (default 0)
//! - `PROPERTY_STORE_SPIKE_INCLUDE_C4` (default off; set `1` to measure C4)
//!
//! Emit one JSON object per measurement line under `--nocapture`:
//! `PROPERTY_STORE_SPIKE {…}`. Fields include `measures`, `hypothesis`, and
//! `feeds_outcome` so lines are self-describing.

use std::env;
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde_json::{json, Value as JsonValue};
use turso_core::{
    Connection, Database, DatabaseOpts, MemoryIO, Numeric, OpenFlags, SqliteDialect, Value,
};

/// Open flags: JSONB bag configs need experimental custom types.

// ---------------------------------------------------------------------------
// Scale
// ---------------------------------------------------------------------------

fn env_usize(name: &str, default: usize) -> usize {
    env::var(name)
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(default)
}

fn env_flag(name: &str) -> bool {
    matches!(
        env::var(name).as_deref(),
        Ok("1") | Ok("true") | Ok("TRUE") | Ok("yes") | Ok("YES")
    )
}

struct Scale {
    types: usize,
    entities_per_type: usize,
    props: usize,
    filter_prop: usize,
}

impl Scale {
    fn from_env() -> Self {
        let types = env_usize("PROPERTY_STORE_SPIKE_TYPES", 50);
        let props = env_usize("PROPERTY_STORE_SPIKE_PROPS", 8).max(1);
        let filter_prop = env_usize("PROPERTY_STORE_SPIKE_FILTER_PROP", 0).min(props - 1);
        Self {
            types,
            entities_per_type: env_usize("PROPERTY_STORE_SPIKE_ENTITIES", 20),
            props,
            filter_prop,
        }
    }

    fn total_entities(&self) -> usize {
        self.types * self.entities_per_type
    }
}

// ---------------------------------------------------------------------------
// Connection helpers
// ---------------------------------------------------------------------------

fn connect(label: &str, custom_types: bool) -> Arc<Connection> {
    let io = Arc::new(MemoryIO::new());
    Database::open_file_with_flags(
        io,
        &format!(":memory:property-store-spike-{label}"),
        OpenFlags::default(),
        DatabaseOpts::new().with_custom_types(custom_types),
        None,
        Arc::new(SqliteDialect),
    )
    .expect("open database")
    .connect()
    .expect("connect")
}

fn exec(conn: &Arc<Connection>, sql: &str) {
    conn.execute(sql)
        .unwrap_or_else(|error| panic!("execute failed: {error}\nsql: {sql}"));
}

fn query_i64(conn: &Arc<Connection>, sql: &str) -> i64 {
    let mut rows = conn
        .prepare(sql)
        .expect("prepare")
        .run_collect_rows()
        .expect("collect");
    match rows.pop().and_then(|row| row.into_iter().next()) {
        Some(Value::Numeric(Numeric::Integer(value))) => value,
        other => panic!("expected integer row for {sql:?}, got {other:?}"),
    }
}

fn schema_counts(conn: &Arc<Connection>) -> (i64, i64, i64) {
    let tables = query_i64(
        conn,
        "SELECT COUNT(*) FROM sqlite_schema WHERE type = 'table' AND name NOT LIKE 'sqlite_%'",
    );
    let indexes = query_i64(
        conn,
        "SELECT COUNT(*) FROM sqlite_schema WHERE type = 'index' AND name NOT LIKE 'sqlite_%'",
    );
    let schema_bytes = query_i64(
        conn,
        "SELECT COALESCE(SUM(LENGTH(sql)), 0) FROM sqlite_schema WHERE sql IS NOT NULL",
    );
    (tables, indexes, schema_bytes)
}

fn timed(work: impl FnOnce()) -> Duration {
    let start = Instant::now();
    work();
    start.elapsed()
}

fn median_ms(samples: &mut [Duration]) -> f64 {
    samples.sort();
    let mid = samples[samples.len() / 2];
    mid.as_secs_f64() * 1000.0
}

fn emit(record: JsonValue) {
    println!("PROPERTY_STORE_SPIKE {record}");
}

// ---------------------------------------------------------------------------
// Layout builders
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Config {
    /// One table per type with fixed property columns.
    C0TypePerTable,
    /// One nodes table; dense property columns.
    C1SharedColumns,
    /// One nodes table; single JSON **TEXT** bag.
    C2SharedJsonBag,
    /// One nodes table; single **JSONB** bag (custom types on).
    C2SharedJsonbBag,
    /// Integer prop_id cells + prop_dict + index (prop_id, value). Open property store.
    C3SharedEav,
    /// String prop_key cells + index (prop_key, value). Head-to-head vs C3 integers.
    C3SharedEavStringKey,
    /// C2 TEXT bag plus expression index on the filter property.
    C4SharedJsonBagExprIndex,
    /// C2j JSONB bag plus expression index on the filter property.
    C4SharedJsonbBagExprIndex,
}

impl Config {
    fn name(self) -> &'static str {
        match self {
            Self::C0TypePerTable => "C0_type_per_table",
            Self::C1SharedColumns => "C1_shared_columns",
            Self::C2SharedJsonBag => "C2_shared_json_bag_text",
            Self::C2SharedJsonbBag => "C2_shared_jsonb_bag",
            Self::C3SharedEav => "C3_shared_cell_prop_id",
            Self::C3SharedEavStringKey => "C3s_shared_cell_prop_key_text",
            Self::C4SharedJsonBagExprIndex => "C4_shared_json_bag_text_expr_index",
            Self::C4SharedJsonbBagExprIndex => "C4_shared_jsonb_bag_expr_index",
        }
    }

    /// One-line physical description for JSON `config_means` and logs.
    fn means(self) -> &'static str {
        match self {
            Self::C0TypePerTable => {
                "one SQL table per type; properties are ordinary columns (type-per-table baseline)"
            }
            Self::C1SharedColumns => {
                "shared nodes+edges tables; properties are ordinary columns"
            }
            Self::C2SharedJsonBag => {
                "shared nodes+edges; all properties in one TEXT JSON column (bag)"
            }
            Self::C2SharedJsonbBag => {
                "shared nodes+edges; all properties in one declared JSONB column (bag)"
            }
            Self::C3SharedEav => {
                "shared nodes+edges; prop_dict + cells keyed by integer prop_id; index (prop_id, value)"
            }
            Self::C3SharedEavStringKey => {
                "shared nodes+edges; cells keyed by prop_key TEXT; index (prop_key, value)"
            }
            Self::C4SharedJsonBagExprIndex => {
                "TEXT bag like C2 plus expression index on json_extract(props, '$.hot')"
            }
            Self::C4SharedJsonbBagExprIndex => {
                "JSONB bag like C2j plus expression index on json_extract(props, '$.hot')"
            }
        }
    }

    fn is_shared(self) -> bool {
        !matches!(self, Self::C0TypePerTable)
    }

    fn needs_custom_types(self) -> bool {
        matches!(
            self,
            Self::C2SharedJsonbBag | Self::C4SharedJsonbBagExprIndex
        )
    }

    fn has_bag_expr_index(self) -> bool {
        matches!(
            self,
            Self::C4SharedJsonBagExprIndex | Self::C4SharedJsonbBagExprIndex
        )
    }

    fn bag_sql_type(self) -> Option<&'static str> {
        match self {
            Self::C2SharedJsonBag | Self::C4SharedJsonBagExprIndex => Some("TEXT"),
            Self::C2SharedJsonbBag | Self::C4SharedJsonbBagExprIndex => Some("JSONB"),
            _ => None,
        }
    }
}

/// Printed once per test that emits metrics so runs are self-documenting.
fn emit_run_banner(test: &str, scale: &Scale, configs: &[Config]) {
    let names: Vec<&str> = configs.iter().map(|c| c.name()).collect();
    emit(json!({
        "phase": "run_banner",
        "test": test,
        "what_this_run_measures": [
            "schema_ms + table_count: cost of creating the layout (H1/H2 type explosion)",
            "filter_eq_ms + filter_plan_uses_index: property equality selectivity (H4)",
            "project_one_ms / project_all_ms: read one prop vs full map (H3)",
            "set_one_ms: single property update cost (H6)",
            "hop_prop_ms: 1-hop join then property filter on neighbor (edge index may dominate)",
        ],
        "decision_outcomes": {
            "A_bag_only": "shared stores + bag enough if filters need no property index",
            "B_cells_for_all": "indexed cells for open property namespace / most filters",
            "C_hybrid": "bag (or columns) for maps + Cell or expr-index for hot filter keys",
        },
        "phase1b_chosen_outcome": "C_hybrid",
        "types": scale.types,
        "entities_per_type": scale.entities_per_type,
        "total_entities": scale.total_entities(),
        "props": scale.props,
        "filter_prop": scale.filter_prop,
        "configs": names,
        "fairness_notes": [
            "C0 project/filter only scan node_0 (one type), not all types",
            "hop_prop_plan_uses_index is any index (often edges), not property-index proof",
            "C3 is a B-tree EAV proxy, not a smoltable/LSM wide-col store",
        ],
    }));
}

fn build_shared_edges(conn: &Arc<Connection>) {
    exec(
        conn,
        "CREATE TABLE edges(\
         id INTEGER PRIMARY KEY, \
         start INTEGER NOT NULL, \
         \"end\" INTEGER NOT NULL)",
    );
    exec(conn, "CREATE INDEX edges_start ON edges(start)");
    exec(conn, "CREATE INDEX edges_end ON edges(\"end\")");
    exec(conn, "CREATE INDEX edges_pair ON edges(start, \"end\")");
}

fn build_schema(conn: &Arc<Connection>, config: Config, scale: &Scale) {
    match config {
        Config::C0TypePerTable => {
            for type_id in 0..scale.types {
                let mut cols = String::from("id INTEGER PRIMARY KEY");
                for prop in 0..scale.props {
                    cols.push_str(&format!(", p{prop} TEXT"));
                }
                exec(conn, &format!("CREATE TABLE node_{type_id}({cols})"));
                exec(
                    conn,
                    &format!("CREATE INDEX node_{type_id}_p0 ON node_{type_id}(p0)"),
                );
            }
        }
        Config::C1SharedColumns => {
            let mut cols = String::from("id INTEGER PRIMARY KEY, type_id INTEGER NOT NULL");
            for prop in 0..scale.props {
                cols.push_str(&format!(", p{prop} TEXT"));
            }
            exec(conn, &format!("CREATE TABLE nodes({cols})"));
            exec(conn, "CREATE INDEX nodes_type ON nodes(type_id)");
            exec(conn, "CREATE INDEX nodes_p0 ON nodes(p0)");
            build_shared_edges(conn);
        }
        Config::C2SharedJsonBag
        | Config::C2SharedJsonbBag
        | Config::C4SharedJsonBagExprIndex
        | Config::C4SharedJsonbBagExprIndex => {
            let bag_ty = config.bag_sql_type().expect("bag config");
            exec(
                conn,
                &format!(
                    "CREATE TABLE nodes(\
                     id INTEGER PRIMARY KEY, \
                     type_id INTEGER NOT NULL, \
                     props {bag_ty} NOT NULL)"
                ),
            );
            exec(conn, "CREATE INDEX nodes_type ON nodes(type_id)");
            if config.has_bag_expr_index() {
                let prop = scale.filter_prop;
                // Hot-key bag index: one expression index per declared hot property.
                // Same json_extract expression for TEXT and JSONB bags (Turso
                // accepts both). Does not scale to open property namespaces.
                exec(
                    conn,
                    &format!(
                        "CREATE INDEX nodes_p{prop}_expr ON nodes(json_extract(props, '$.p{prop}'))"
                    ),
                );
            }
            build_shared_edges(conn);
        }
        Config::C3SharedEav => {
            exec(
                conn,
                "CREATE TABLE nodes(\
                 id INTEGER PRIMARY KEY, \
                 type_id INTEGER NOT NULL)",
            );
            // Dictionary: name → prop_id once; cells store integers only.
            exec(
                conn,
                "CREATE TABLE prop_dict(\
                 prop_id INTEGER PRIMARY KEY, \
                 name TEXT NOT NULL COLLATE NOCASE UNIQUE, \
                 value_type TEXT NOT NULL)",
            );
            exec(
                conn,
                "CREATE TABLE node_props(\
                 node_id INTEGER NOT NULL, \
                 prop_id INTEGER NOT NULL, \
                 value TEXT, \
                 PRIMARY KEY(node_id, prop_id))",
            );
            exec(conn, "CREATE INDEX nodes_type ON nodes(type_id)");
            exec(
                conn,
                "CREATE INDEX node_props_by_kv ON node_props(prop_id, value)",
            );
            build_shared_edges(conn);
        }
        Config::C3SharedEavStringKey => {
            exec(
                conn,
                "CREATE TABLE nodes(\
                 id INTEGER PRIMARY KEY, \
                 type_id INTEGER NOT NULL)",
            );
            exec(
                conn,
                "CREATE TABLE node_props(\
                 node_id INTEGER NOT NULL, \
                 prop_key TEXT NOT NULL, \
                 value TEXT, \
                 PRIMARY KEY(node_id, prop_key))",
            );
            exec(conn, "CREATE INDEX nodes_type ON nodes(type_id)");
            exec(
                conn,
                "CREATE INDEX node_props_by_kv ON node_props(prop_key, value)",
            );
            build_shared_edges(conn);
        }
    }
}

fn load_shared_edges(conn: &Arc<Connection>, scale: &Scale) {
    // Directed chain over all entity ids: i -> i+1. Enough for a 1-hop join.
    let total = scale.total_entities();
    if total < 2 {
        return;
    }
    for id in 0..(total - 1) {
        let end = id + 1;
        exec(
            conn,
            &format!("INSERT INTO edges(id, start, \"end\") VALUES ({id}, {id}, {end})"),
        );
    }
}

fn load_data(conn: &Arc<Connection>, config: Config, scale: &Scale) {
    match config {
        Config::C0TypePerTable => {
            for type_id in 0..scale.types {
                for local in 0..scale.entities_per_type {
                    let id = type_id * scale.entities_per_type + local;
                    let mut values = format!("{id}");
                    for prop in 0..scale.props {
                        values.push_str(&format!(", 't{type_id}-e{local}-p{prop}'"));
                    }
                    exec(
                        conn,
                        &format!("INSERT INTO node_{type_id} VALUES ({values})"),
                    );
                }
            }
        }
        Config::C1SharedColumns => {
            for type_id in 0..scale.types {
                for local in 0..scale.entities_per_type {
                    let id = type_id * scale.entities_per_type + local;
                    let mut values = format!("{id}, {type_id}");
                    for prop in 0..scale.props {
                        values.push_str(&format!(", 't{type_id}-e{local}-p{prop}'"));
                    }
                    exec(conn, &format!("INSERT INTO nodes VALUES ({values})"));
                }
            }
            load_shared_edges(conn, scale);
        }
        Config::C2SharedJsonBag
        | Config::C2SharedJsonbBag
        | Config::C4SharedJsonBagExprIndex
        | Config::C4SharedJsonbBagExprIndex => {
            for type_id in 0..scale.types {
                for local in 0..scale.entities_per_type {
                    let id = type_id * scale.entities_per_type + local;
                    let mut bag = String::from("{");
                    for prop in 0..scale.props {
                        if prop > 0 {
                            bag.push(',');
                        }
                        bag.push_str(&format!("\"p{prop}\":\"t{type_id}-e{local}-p{prop}\""));
                    }
                    bag.push('}');
                    let escaped = bag.replace('\'', "''");
                    // jsonb() coerces text into declared JSONB columns when custom types are on.
                    let props_expr = if config.needs_custom_types() {
                        format!("jsonb('{escaped}')")
                    } else {
                        format!("'{escaped}'")
                    };
                    exec(
                        conn,
                        &format!(
                            "INSERT INTO nodes(id, type_id, props) VALUES ({id}, {type_id}, {props_expr})"
                        ),
                    );
                }
            }
            load_shared_edges(conn, scale);
        }
        Config::C3SharedEav => {
            for prop in 0..scale.props {
                exec(
                    conn,
                    &format!(
                        "INSERT INTO prop_dict(prop_id, name, value_type) \
                         VALUES ({prop}, 'p{prop}', 'text')"
                    ),
                );
            }
            for type_id in 0..scale.types {
                for local in 0..scale.entities_per_type {
                    let id = type_id * scale.entities_per_type + local;
                    exec(
                        conn,
                        &format!("INSERT INTO nodes(id, type_id) VALUES ({id}, {type_id})"),
                    );
                    for prop in 0..scale.props {
                        let value = format!("t{type_id}-e{local}-p{prop}");
                        exec(
                            conn,
                            &format!(
                                "INSERT INTO node_props(node_id, prop_id, value) \
                                 VALUES ({id}, {prop}, '{value}')"
                            ),
                        );
                    }
                }
            }
            load_shared_edges(conn, scale);
        }
        Config::C3SharedEavStringKey => {
            for type_id in 0..scale.types {
                for local in 0..scale.entities_per_type {
                    let id = type_id * scale.entities_per_type + local;
                    exec(
                        conn,
                        &format!("INSERT INTO nodes(id, type_id) VALUES ({id}, {type_id})"),
                    );
                    for prop in 0..scale.props {
                        let value = format!("t{type_id}-e{local}-p{prop}");
                        // Realistic-ish key length: prefix + property number.
                        exec(
                            conn,
                            &format!(
                                "INSERT INTO node_props(node_id, prop_key, value) \
                                 VALUES ({id}, 'property_name_{prop}', '{value}')"
                            ),
                        );
                    }
                }
            }
            load_shared_edges(conn, scale);
        }
    }
}

// ---------------------------------------------------------------------------
// Workloads
// ---------------------------------------------------------------------------

fn sql_project_one(config: Config, scale: &Scale) -> String {
    let prop = scale.filter_prop;
    match config {
        Config::C0TypePerTable => {
            // Representative single-type project (type 0); full ontology would UNION.
            format!("SELECT p{prop} FROM node_0")
        }
        Config::C1SharedColumns => format!("SELECT p{prop} FROM nodes"),
        Config::C2SharedJsonBag
        | Config::C2SharedJsonbBag
        | Config::C4SharedJsonBagExprIndex
        | Config::C4SharedJsonbBagExprIndex => {
            format!("SELECT json_extract(props, '$.p{prop}') FROM nodes")
        }
        Config::C3SharedEav => format!("SELECT value FROM node_props WHERE prop_id = {prop}"),
        Config::C3SharedEavStringKey => {
            format!("SELECT value FROM node_props WHERE prop_key = 'property_name_{prop}'")
        }
    }
}

fn sql_project_all(config: Config, scale: &Scale) -> String {
    match config {
        Config::C0TypePerTable => {
            let cols = (0..scale.props)
                .map(|prop| format!("p{prop}"))
                .collect::<Vec<_>>()
                .join(", ");
            format!("SELECT {cols} FROM node_0")
        }
        Config::C1SharedColumns => {
            let cols = (0..scale.props)
                .map(|prop| format!("p{prop}"))
                .collect::<Vec<_>>()
                .join(", ");
            format!("SELECT {cols} FROM nodes")
        }
        Config::C2SharedJsonBag
        | Config::C2SharedJsonbBag
        | Config::C4SharedJsonBagExprIndex
        | Config::C4SharedJsonbBagExprIndex => "SELECT props FROM nodes".to_owned(),
        Config::C3SharedEav => {
            "SELECT node_id, prop_id, value FROM node_props ORDER BY node_id, prop_id".to_owned()
        }
        Config::C3SharedEavStringKey => {
            "SELECT node_id, prop_key, value FROM node_props ORDER BY node_id, prop_key".to_owned()
        }
    }
}

fn sql_filter_eq(config: Config, scale: &Scale) -> String {
    // Unique value hits one row: fair index probe.
    let target = format!("t0-e0-p{}", scale.filter_prop);
    let prop = scale.filter_prop;
    match config {
        Config::C0TypePerTable => format!("SELECT id FROM node_0 WHERE p{prop} = '{target}'"),
        Config::C1SharedColumns => format!("SELECT id FROM nodes WHERE p{prop} = '{target}'"),
        Config::C2SharedJsonBag
        | Config::C2SharedJsonbBag
        | Config::C4SharedJsonBagExprIndex
        | Config::C4SharedJsonbBagExprIndex => format!(
            "SELECT id FROM nodes WHERE json_extract(props, '$.p{prop}') = '{target}'"
        ),
        Config::C3SharedEav => format!(
            "SELECT node_id FROM node_props WHERE prop_id = {prop} AND value = '{target}'"
        ),
        Config::C3SharedEavStringKey => format!(
            "SELECT node_id FROM node_props WHERE prop_key = 'property_name_{prop}' AND value = '{target}'"
        ),
    }
}

/// Two-property AND (open multi-predicate filter).
fn sql_filter_and_two(config: Config, scale: &Scale) -> Option<String> {
    if scale.props < 2 {
        return None;
    }
    let p0 = 0usize;
    let p1 = scale.props - 1; // first + last: open subset of property space
    let t0 = format!("t0-e0-p{p0}");
    let t1 = format!("t0-e0-p{p1}");
    let sql = match config {
        Config::C1SharedColumns => {
            format!("SELECT id FROM nodes WHERE p{p0} = '{t0}' AND p{p1} = '{t1}'")
        }
        Config::C2SharedJsonBag
        | Config::C2SharedJsonbBag
        | Config::C4SharedJsonBagExprIndex
        | Config::C4SharedJsonbBagExprIndex => format!(
            "SELECT id FROM nodes WHERE json_extract(props, '$.p{p0}') = '{t0}' \
             AND json_extract(props, '$.p{p1}') = '{t1}'"
        ),
        Config::C3SharedEav => format!(
            "SELECT a.node_id FROM node_props a \
             JOIN node_props b ON b.node_id = a.node_id \
             WHERE a.prop_id = {p0} AND a.value = '{t0}' \
               AND b.prop_id = {p1} AND b.value = '{t1}'"
        ),
        Config::C3SharedEavStringKey => format!(
            "SELECT a.node_id FROM node_props a \
             JOIN node_props b ON b.node_id = a.node_id \
             WHERE a.prop_key = 'property_name_{p0}' AND a.value = '{t0}' \
               AND b.prop_key = 'property_name_{p1}' AND b.value = '{t1}'"
        ),
        Config::C0TypePerTable => {
            format!("SELECT id FROM node_0 WHERE p{p0} = '{t0}' AND p{p1} = '{t1}'")
        }
    };
    Some(sql)
}

fn sql_set_one(config: Config, scale: &Scale) -> String {
    let prop = scale.filter_prop;
    match config {
        Config::C0TypePerTable => format!("UPDATE node_0 SET p{prop} = 'updated' WHERE id = 0"),
        Config::C1SharedColumns => format!("UPDATE nodes SET p{prop} = 'updated' WHERE id = 0"),
        Config::C2SharedJsonBag
        | Config::C2SharedJsonbBag
        | Config::C4SharedJsonBagExprIndex
        | Config::C4SharedJsonbBagExprIndex => {
            // json_set works on both TEXT and JSONB bags in Turso.
            format!("UPDATE nodes SET props = json_set(props, '$.p{prop}', 'updated') WHERE id = 0")
        }
        Config::C3SharedEav => format!(
            "UPDATE node_props SET value = 'updated' WHERE node_id = 0 AND prop_id = {prop}"
        ),
        Config::C3SharedEavStringKey => format!(
            "UPDATE node_props SET value = 'updated' WHERE node_id = 0 AND prop_key = 'property_name_{prop}'"
        ),
    }
}

/// 1-hop from a fixed start, then filter the neighbor on the property.
/// Shared stores only. Models graph expand + property predicate on intermediate.
fn sql_hop_prop(config: Config, scale: &Scale) -> Option<String> {
    if !config.is_shared() {
        return None;
    }
    let target = format!("t0-e1-p{}", scale.filter_prop);
    let prop = scale.filter_prop;
    let sql = match config {
        Config::C1SharedColumns => format!(
            "SELECT n.id FROM edges e \
             JOIN nodes n ON n.id = e.\"end\" \
             WHERE e.start = 0 AND n.p{prop} = '{target}'"
        ),
        Config::C2SharedJsonBag
        | Config::C2SharedJsonbBag
        | Config::C4SharedJsonBagExprIndex
        | Config::C4SharedJsonbBagExprIndex => format!(
            "SELECT n.id FROM edges e \
             JOIN nodes n ON n.id = e.\"end\" \
             WHERE e.start = 0 AND json_extract(n.props, '$.p{prop}') = '{target}'"
        ),
        Config::C3SharedEav => format!(
            "SELECT n.id FROM edges e \
             JOIN nodes n ON n.id = e.\"end\" \
             JOIN node_props p ON p.node_id = n.id AND p.prop_id = {prop} \
             WHERE e.start = 0 AND p.value = '{target}'"
        ),
        Config::C3SharedEavStringKey => format!(
            "SELECT n.id FROM edges e \
             JOIN nodes n ON n.id = e.\"end\" \
             JOIN node_props p ON p.node_id = n.id AND p.prop_key = 'property_name_{prop}' \
             WHERE e.start = 0 AND p.value = '{target}'"
        ),
        Config::C0TypePerTable => return None,
    };
    Some(sql)
}

fn run_query_iters(conn: &Arc<Connection>, sql: &str, iters: usize) -> (f64, i64) {
    let warm_rows = conn
        .prepare(sql)
        .expect("prepare warm")
        .run_collect_rows()
        .expect("warm collect");
    let row_count = warm_rows.len() as i64;

    let mut samples = Vec::with_capacity(iters);
    for _ in 0..iters {
        samples.push(timed(|| {
            conn.prepare(sql)
                .expect("prepare")
                .run_collect_rows()
                .expect("collect");
        }));
    }
    (median_ms(&mut samples), row_count)
}

fn explain_uses_index(conn: &Arc<Connection>, sql: &str) -> bool {
    let plan_sql = format!("EXPLAIN QUERY PLAN {sql}");
    let rows = conn
        .prepare(&plan_sql)
        .expect("explain prepare")
        .run_collect_rows()
        .expect("explain collect");
    let text = rows
        .iter()
        .flat_map(|row| row.iter())
        .filter_map(|value| match value {
            Value::Text(text) => Some(text.as_str().to_ascii_lowercase()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join(" ");
    text.contains("using index") || text.contains("using covering index")
}

fn reset_entity_zero(conn: &Arc<Connection>, config: Config, scale: &Scale) {
    let prop = scale.filter_prop;
    match config {
        Config::C0TypePerTable => {
            exec(
                conn,
                &format!("UPDATE node_0 SET p{prop} = 'seed' WHERE id = 0"),
            );
        }
        Config::C1SharedColumns => {
            exec(
                conn,
                &format!("UPDATE nodes SET p{prop} = 'seed' WHERE id = 0"),
            );
        }
        Config::C2SharedJsonBag
        | Config::C2SharedJsonbBag
        | Config::C4SharedJsonBagExprIndex
        | Config::C4SharedJsonbBagExprIndex => {
            exec(
                conn,
                &format!(
                    "UPDATE nodes SET props = json_set(props, '$.p{prop}', 'seed') WHERE id = 0"
                ),
            );
        }
        Config::C3SharedEav => {
            exec(
                conn,
                &format!(
                    "UPDATE node_props SET value = 'seed' WHERE node_id = 0 AND prop_id = {prop}"
                ),
            );
        }
        Config::C3SharedEavStringKey => {
            exec(
                conn,
                &format!(
                    "UPDATE node_props SET value = 'seed' WHERE node_id = 0 AND prop_key = 'property_name_{prop}'"
                ),
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Experiments
// ---------------------------------------------------------------------------

fn measure_schema_only(config: Config, scale: &Scale) {
    let conn = connect(config.name(), config.needs_custom_types());
    let schema_ms = timed(|| build_schema(&conn, config, scale)).as_secs_f64() * 1000.0;
    let (tables, indexes, schema_bytes) = schema_counts(&conn);
    emit(json!({
        "phase": "schema",
        "measures": "CREATE TABLE/INDEX only; no rows loaded",
        "hypothesis": ["H1", "H2"],
        "feeds_outcome": "If C0 table_count and schema_ms explode while C1–C3 stay flat, shared stores are required before bag vs cell matters",
        "config": config.name(),
        "config_means": config.means(),
        "bag_sql_type": config.bag_sql_type(),
        "needs_custom_types": config.needs_custom_types(),
        "types": scale.types,
        "props": scale.props,
        "schema_ms": schema_ms,
        "table_count": tables,
        "index_count": indexes,
        "schema_sql_bytes": schema_bytes,
    }));
}

fn measure_full(config: Config, scale: &Scale) {
    let conn = connect(
        &format!("{}-full", config.name()),
        config.needs_custom_types(),
    );
    let schema_ms = timed(|| build_schema(&conn, config, scale)).as_secs_f64() * 1000.0;
    let load_ms = timed(|| load_data(&conn, config, scale)).as_secs_f64() * 1000.0;
    let (tables, indexes, schema_bytes) = schema_counts(&conn);

    let project_one_sql = sql_project_one(config, scale);
    let project_all_sql = sql_project_all(config, scale);
    let filter_sql = sql_filter_eq(config, scale);
    let set_sql = sql_set_one(config, scale);

    let (project_one_ms, project_one_rows) = run_query_iters(&conn, &project_one_sql, 5);
    let (project_all_ms, project_all_rows) = run_query_iters(&conn, &project_all_sql, 5);
    let (filter_ms, filter_rows) = run_query_iters(&conn, &filter_sql, 5);
    let plan_uses_index = explain_uses_index(&conn, &filter_sql);

    let mut set_samples = Vec::with_capacity(5);
    for _ in 0..5 {
        reset_entity_zero(&conn, config, scale);
        set_samples.push(timed(|| exec(&conn, &set_sql)));
    }
    let set_one_ms = median_ms(&mut set_samples);

    // How to read this config against the decision gate (heuristic labels only).
    let gate_read = if !config.is_shared() {
        "baseline_type_per_table_only; compare table_count/schema_ms to shared, not query_ms to C1–C4 full scans"
    } else if matches!(config, Config::C2SharedJsonBag | Config::C2SharedJsonbBag)
        && !plan_uses_index
    {
        "feeds_A_if_filter_eq_acceptable; feeds_C_or_B_if_filter_eq_slow_vs_C3_or_C4"
    } else if config.has_bag_expr_index() && plan_uses_index {
        "feeds_C_hybrid: bag map project + expr index on hot key (TEXT or JSONB bag)"
    } else if matches!(config, Config::C3SharedEav) && plan_uses_index {
        "open_cell_prop_id: index (prop_id,value) for arbitrary properties; compare to C3s string keys and bags"
    } else if matches!(config, Config::C3SharedEavStringKey) && plan_uses_index {
        "open_cell_prop_key_text: baseline for dictionary; expect higher cost than integer prop_id"
    } else {
        "shared_columns_or_indexed_path; compare filter_eq and project_all to bag"
    };

    let mut record = json!({
        "phase": "full",
        "measures": "schema + load + filter equality + project one/all + set one property (+ hop+prop on shared)",
        "hypothesis": ["H1", "H2", "H3", "H4", "H6"],
        "feeds_outcome": gate_read,
        "config": config.name(),
        "config_means": config.means(),
        "types": scale.types,
        "entities_per_type": scale.entities_per_type,
        "total_entities": scale.total_entities(),
        "props": scale.props,
        "filter_prop": scale.filter_prop,
        "bag_sql_type": config.bag_sql_type(),
        "needs_custom_types": config.needs_custom_types(),
        "schema_ms": schema_ms,
        "load_ms": load_ms,
        "table_count": tables,
        "index_count": indexes,
        "schema_sql_bytes": schema_bytes,
        "project_one_ms": project_one_ms,
        "project_one_rows": project_one_rows,
        "project_one_means": "read one property from scanned rows (C0: only node_0)",
        "project_all_ms": project_all_ms,
        "project_all_rows": project_all_rows,
        "project_all_means": "read full property map (bag column) or all EAV cells",
        "filter_eq_ms": filter_ms,
        "filter_eq_rows": filter_rows,
        "filter_eq_means": "SELECT ids WHERE property equals one unique value (H4)",
        "filter_plan_uses_index": plan_uses_index,
        "filter_plan_uses_index_means": "EXPLAIN QUERY PLAN mentions USING INDEX on the property filter path",
        "set_one_ms": set_one_ms,
        "set_one_means": "UPDATE one property on entity id=0 (H6; bag may rewrite whole JSON)",
        "timing_method": "median of 5 runs after 1 warm run; wall ms",
    });

    if let Some(hop_sql) = sql_hop_prop(config, scale) {
        let (hop_ms, hop_rows) = run_query_iters(&conn, &hop_sql, 5);
        let hop_uses_index = explain_uses_index(&conn, &hop_sql);
        let obj = record.as_object_mut().expect("object");
        obj.insert("hop_prop_ms".to_owned(), json!(hop_ms));
        obj.insert("hop_prop_rows".to_owned(), json!(hop_rows));
        obj.insert("hop_prop_plan_uses_index".to_owned(), json!(hop_uses_index));
        obj.insert(
            "hop_prop_means".to_owned(),
            json!("1-hop edges.start=0 then filter neighbor property; plan index often edges_start, not property index"),
        );
    }

    if let Some(and_sql) = sql_filter_and_two(config, scale) {
        let (and_ms, and_rows) = run_query_iters(&conn, &and_sql, 5);
        let and_uses_index = explain_uses_index(&conn, &and_sql);
        let obj = record.as_object_mut().expect("object");
        obj.insert("filter_and2_ms".to_owned(), json!(and_ms));
        obj.insert("filter_and2_rows".to_owned(), json!(and_rows));
        obj.insert(
            "filter_and2_plan_uses_index".to_owned(),
            json!(and_uses_index),
        );
        obj.insert(
            "filter_and2_means".to_owned(),
            json!("AND of first and last property equalities (open multi-predicate subset)"),
        );
    }

    // Cell density: how many property rows for open store configs.
    if matches!(config, Config::C3SharedEav | Config::C3SharedEavStringKey) {
        let cell_rows = query_i64(&conn, "SELECT COUNT(*) FROM node_props");
        let obj = record.as_object_mut().expect("object");
        obj.insert("cell_row_count".to_owned(), json!(cell_rows));
        obj.insert(
            "cell_row_count_means".to_owned(),
            json!("entities × props; open store cost driver"),
        );
    }

    emit(record);
}

fn configs_for_full_run() -> Vec<Config> {
    let mut configs = vec![
        Config::C0TypePerTable,
        Config::C1SharedColumns,
        Config::C2SharedJsonBag,
        Config::C2SharedJsonbBag,
        Config::C3SharedEav,
        Config::C3SharedEavStringKey,
    ];
    if env_flag("PROPERTY_STORE_SPIKE_INCLUDE_C4") {
        configs.push(Config::C4SharedJsonBagExprIndex);
        configs.push(Config::C4SharedJsonbBagExprIndex);
    }
    configs
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[test]
fn schema_scaling_type_per_table_vs_shared() {
    // Outcome signal: C0 table_count grows with TYPES; C1–C3 stay O(1) tables.
    let scale = Scale::from_env();
    let configs = [
        Config::C0TypePerTable,
        Config::C1SharedColumns,
        Config::C2SharedJsonBag,
        Config::C2SharedJsonbBag,
        Config::C3SharedEav,
        Config::C3SharedEavStringKey,
    ];
    emit_run_banner("schema_scaling_type_per_table_vs_shared", &scale, &configs);
    for config in configs {
        measure_schema_only(config, &scale);
    }

    // Shared layouts must stay near-constant table count regardless of type count.
    let shared = connect("assert-shared", false);
    build_schema(&shared, Config::C2SharedJsonBag, &scale);
    let (tables, _, _) = schema_counts(&shared);
    assert!(
        tables <= 4,
        "shared JSON bag layout grew tables unexpectedly: {tables}"
    );

    let jsonb = connect("assert-jsonb-schema", true);
    build_schema(&jsonb, Config::C2SharedJsonbBag, &scale);
    let (jsonb_tables, _, _) = schema_counts(&jsonb);
    assert_eq!(
        jsonb_tables, tables,
        "JSONB bag table count matches TEXT bag"
    );

    let per_type = connect("assert-c0", false);
    build_schema(&per_type, Config::C0TypePerTable, &scale);
    let (c0_tables, _, _) = schema_counts(&per_type);
    assert_eq!(
        c0_tables as usize, scale.types,
        "C0 should create one table per type (H1 type explosion)"
    );
    assert!(
        c0_tables > tables,
        "type-per-table must dominate shared table count (H2)"
    );
}

#[test]
fn property_access_methods_on_shared_and_baseline() {
    // Outcome signal: compare filter_eq_ms and filter_plan_uses_index across
    // C2 (bag), C3 (cells), C4 (bag+hot index) and project_all_ms (map cost).
    let scale = Scale::from_env();
    let configs = configs_for_full_run();
    emit_run_banner(
        "property_access_methods_on_shared_and_baseline",
        &scale,
        &configs,
    );
    for config in configs {
        measure_full(config, &scale);
    }

    // Correctness: filter returns exactly one row for the seeded unique value.
    let conn = connect("assert-filter", false);
    build_schema(&conn, Config::C3SharedEav, &scale);
    load_data(&conn, Config::C3SharedEav, &scale);
    let rows = query_i64(
        &conn,
        &format!(
            "SELECT COUNT(*) FROM node_props WHERE prop_id = {} AND value = 't0-e0-p{}'",
            scale.filter_prop, scale.filter_prop
        ),
    );
    assert_eq!(rows, 1, "filter target must be unique");
    assert!(
        explain_uses_index(
            &conn,
            &format!(
                "SELECT node_id FROM node_props WHERE prop_id = {} AND value = 't0-e0-p{}'",
                scale.filter_prop, scale.filter_prop
            )
        ),
        "integer prop_id open filter must use node_props_by_kv"
    );
}

#[test]
fn eav_filter_plan_can_use_property_index() {
    // Correctness gate for Outcome B/C cell path: EXPLAIN must use the value index.
    let scale = Scale {
        types: 8,
        entities_per_type: 16,
        props: 4,
        filter_prop: 0,
    };
    let conn = connect("assert-eav-plan", false);
    build_schema(&conn, Config::C3SharedEav, &scale);
    load_data(&conn, Config::C3SharedEav, &scale);
    let sql = sql_filter_eq(Config::C3SharedEav, &scale);
    assert!(
        explain_uses_index(&conn, &sql),
        "C3 integer prop_id filter must use node_props_by_kv; without this open Cell path fails"
    );
    let and_sql = sql_filter_and_two(Config::C3SharedEav, &scale).expect("and sql");
    assert_eq!(
        query_i64(&conn, &format!("SELECT COUNT(*) FROM ({and_sql})")),
        1
    );
}

#[test]
fn hop_then_property_filter_returns_one_neighbor() {
    // Correctness for hop+prop SQL shape only (not a timing gate).
    let scale = Scale {
        types: 4,
        entities_per_type: 8,
        props: 4,
        filter_prop: 0,
    };
    // Chain: 0 -> 1. Neighbor 1 has type 0 local 1 → value t0-e1-p0.
    for config in [
        Config::C1SharedColumns,
        Config::C2SharedJsonBag,
        Config::C2SharedJsonbBag,
        Config::C3SharedEav,
    ] {
        let conn = connect(
            &format!("hop-{}", config.name()),
            config.needs_custom_types(),
        );
        build_schema(&conn, config, &scale);
        load_data(&conn, config, &scale);
        let sql = sql_hop_prop(config, &scale).expect("shared config has hop SQL");
        let rows = conn
            .prepare(&sql)
            .expect("prepare hop")
            .run_collect_rows()
            .expect("hop collect");
        assert_eq!(
            rows.len(),
            1,
            "{} hop+prop should hit neighbor id 1",
            config.name()
        );
        assert_eq!(
            rows[0][0],
            Value::from_i64(1),
            "{} hop+prop should return neighbor 1",
            config.name()
        );
    }
}

#[test]
fn bag_expression_index_can_be_created_for_hot_key() {
    // Outcome C hybrid signal: TEXT and JSONB bags + expr index use filter index.
    let scale = Scale {
        types: 4,
        entities_per_type: 8,
        props: 4,
        filter_prop: 0,
    };
    for config in [
        Config::C4SharedJsonBagExprIndex,
        Config::C4SharedJsonbBagExprIndex,
    ] {
        let conn = connect(
            &format!("assert-{}", config.name()),
            config.needs_custom_types(),
        );
        build_schema(&conn, config, &scale);
        load_data(&conn, config, &scale);
        let sql = sql_filter_eq(config, &scale);
        let rows = query_i64(&conn, &format!("SELECT COUNT(*) FROM ({sql})"));
        assert_eq!(rows, 1, "{} filter must hit one row", config.name());
        let uses = explain_uses_index(&conn, &sql);
        emit(json!({
            "phase": "c4_plan_check",
            "measures": "equality filter on bag property with expression index present",
            "hypothesis": ["H4", "H5"],
            "feeds_outcome": "C_hybrid when filter_plan_uses_index is true and project_all stays bag-cheap",
            "config": config.name(),
            "config_means": config.means(),
            "bag_sql_type": config.bag_sql_type(),
            "needs_custom_types": config.needs_custom_types(),
            "filter_plan_uses_index": uses,
            "filter_plan_uses_index_means": "true means planner used the hot-key expression index on the bag",
        }));
        assert!(
            uses,
            "{} hybrid path requires EXPLAIN to use the bag expression index on the filter property",
            config.name()
        );
    }
}

#[test]
fn jsonb_bag_without_index_does_not_claim_property_index() {
    // Control: JSONB bag alone is still a scan for equality (H4), like TEXT bag.
    let scale = Scale {
        types: 4,
        entities_per_type: 8,
        props: 4,
        filter_prop: 0,
    };
    let conn = connect("assert-c2j", true);
    build_schema(&conn, Config::C2SharedJsonbBag, &scale);
    load_data(&conn, Config::C2SharedJsonbBag, &scale);
    let sql = sql_filter_eq(Config::C2SharedJsonbBag, &scale);
    assert_eq!(
        query_i64(&conn, &format!("SELECT COUNT(*) FROM ({sql})")),
        1
    );
    let uses = explain_uses_index(&conn, &sql);
    emit(json!({
        "phase": "c2j_plan_check",
        "config": Config::C2SharedJsonbBag.name(),
        "bag_sql_type": "JSONB",
        "filter_plan_uses_index": uses,
        "feeds_outcome": "JSONB without expr index should not use a property-value index",
    }));
    assert!(
        !uses,
        "C2j JSONB bag without expression index must not report property-index use"
    );
}
