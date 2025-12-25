use std::io::Write;

use rusqlite::hooks::{AuthContext, Authorization};
use serde_json::{Map, Value};
use zet::config::Config;
use zet::core::db::DB;
use zet::preamble::*;

fn disallow_non_selects_authorizer<'r>(ctx: AuthContext<'r>) -> Authorization {
    log::debug!("actions: {:?}", ctx.action);
    match ctx.action {
        rusqlite::hooks::AuthAction::Select => Authorization::Allow,
        rusqlite::hooks::AuthAction::Read {
            table_name: _,
            column_name: _,
        } => Authorization::Allow,
        rusqlite::hooks::AuthAction::Function { function_name: _ } => Authorization::Allow,
        rusqlite::hooks::AuthAction::Pragma {
            pragma_name: _,
            pragma_value: _,
        } => Authorization::Allow,
        _ => Authorization::Deny,
    }
}

// we should only allow
pub fn handle_command(config: Config, query: String) -> Result<()> {
    let root = &config.root;
    let db_path = zet::core::db_dir(root);
    let db = DB::open(db_path)?;

    db.authorizer(Some(disallow_non_selects_authorizer));

    let res = {
        let mut query = db.prepare(&query)?;

        let _columns: Vec<String> = query.column_names().iter().map(|&s| s.to_owned()).collect();

        let res: Vec<serde_json::Value> = query
            .query_map([], |_r| {
                let res = Map::new();

                // TODO figure out how this should work
                // for (i, col) in columns.iter().enumerate() {
                //     let str_val = r.get_ref(i)?.data_type();
                //     log::debug!("query result type: {} {:?}", i, str_val);
                // }
                // for (i, col) in columns.iter().enumerate() {

                //     let value = r.get_ref(i)?;
                //     match (value.data_type()) {

                //     }

                //     let val = r.get::<usize, serde_json::Value>(i)?;
                //     res.insert((*col).to_owned(), val);
                // }

                Ok(Value::Object(res))
            })?
            .map(|f| f.map_err(From::from))
            .collect::<Result<Vec<serde_json::Value>>>()?;

        // disable auth again
        db.authorizer::<fn(AuthContext<'_>) -> Authorization>(None);

        res
    };

    let res = Value::Array(res);

    let res_str = serde_json::to_string_pretty(&res)?;

    std::io::stdout().write_all(res_str.as_bytes())?;

    Ok(())
}
