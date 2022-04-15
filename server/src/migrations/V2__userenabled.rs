use barrel::{Migration, types};
use barrel::backend::Pg;
use barrel::types::WrappedDefault;

pub fn migration() -> String {
    let mut m = Migration::new();
    m.change_table("app_users", |t| {
        t.add_column("enabled", types::boolean().nullable(false)
            .default(WrappedDefault::Boolean(false)));
    });
    m.create_table("verification_tokens", |t| {
        t.add_column("id", types::serial().primary(true));
        t.add_column("user_id", types::integer().nullable(false));
        t.add_column("token", types::varchar(255).nullable(false));
        t.add_column("expiry_date", types::datetime().nullable(false));
        t.add_foreign_key(&["user_id"], "app_users", &["id"]);
    });
    m.make::<Pg>()
}