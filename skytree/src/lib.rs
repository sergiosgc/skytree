use handlebars::Handlebars;
pub mod schema;
pub mod skytree;
#[derive(Debug, Default, Clone)]
pub struct AppData<'a> {
    pub handlebars: Handlebars<'a>,
}
impl negotiated::AppData for AppData<'_> {
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