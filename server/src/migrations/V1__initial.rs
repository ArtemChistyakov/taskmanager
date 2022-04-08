use std::time::SystemTime;

use barrel::{Migration, types};
use barrel::backend::Pg;
use barrel::functions::AutogenFunction;
use barrel::types::WrappedDefault;
use mobc_postgres::tokio_postgres::types::ToSql;

pub fn migration() -> String {
    let mut m = Migration::new();

    m.create_table("app_users", |t| {
        t.add_column("id", types::serial().primary(true));
        t.add_column("email", types::varchar(255).unique(true).nullable(false));
        t.add_column("first_name", types::varchar(255).nullable(true));
        t.add_column("last_name", types::varchar(255).nullable(true));
        t.add_column("pwd", types::varchar(255).nullable(false));
        t.add_column("created_at", types::datetime()
            .default(WrappedDefault::Function(AutogenFunction::CurrentTimestamp)));
    });

    m.create_table("projects", |t| {
        t.add_column("id", types::serial().primary(true));
        t.add_column("title", types::varchar(255).nullable(false));
        t.add_column("description", types::varchar(255).nullable(true));
        t.add_column("created_at", types::datetime()
            .default(WrappedDefault::Function(AutogenFunction::CurrentTimestamp)));
    });

    m.create_table("tasks", |t| {
        t.add_column("id", types::serial().primary(true));
        t.add_column("title", types::varchar(255).nullable(false));
        t.add_column("description", types::varchar(255).nullable(true));
        t.add_column("user_id", types::integer().nullable(false));
        t.add_column("project_id", types::integer().nullable(true));
        t.add_column("created_at", types::datetime()
            .default(WrappedDefault::Function(AutogenFunction::CurrentTimestamp)));
        t.add_foreign_key(&["user_id"], "app_users", &["id"]);
        t.add_foreign_key(&["project_id"], "projects", &["id"]);
    });

    m.create_table("users_projects", |t| {
        t.add_column("user_id", types::integer().nullable(false));
        t.add_column("project_id", types::integer().nullable(false));
        t.set_primary_key(&["user_id", "project_id"]);
        t.add_foreign_key(&["user_id"], "app_users", &["id"]);
        t.add_foreign_key(&["project_id"], "projects", &["id"]);
    });
    m.make::<Pg>()
}