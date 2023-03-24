use diesel::{Connection, SqliteConnection};
use handlebars::Handlebars;
pub mod schema;
pub mod skytree;
#[derive(Debug, Default, Clone)]
pub struct AppData<'a> {
    pub handlebars: Handlebars<'a>,
}
impl rest::DbFactory<SqliteConnection> for AppData<'_> {
    fn db(&self) -> SqliteConnection {
        SqliteConnection::establish(&std::env::var("DATABASE_URL").expect("DATABASE_URL must be set")).unwrap()
    }
}
impl negotiated::HandlebarsFactory for AppData<'static> {
    fn handlebars(&self) -> &Handlebars {
        &self.handlebars
    }

}
pub struct Config {
    pub template_dir: &'static str,
}
impl Config {
    pub fn get_template_dir() -> &'static str {
        unsafe {
            CONFIG.template_dir
        }
    }
}

pub static mut CONFIG: Config = Config { 
    template_dir: "",
};