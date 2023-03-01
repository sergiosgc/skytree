use handlebars::Handlebars;

pub mod skytree;
pub mod negotiated;
#[derive(Debug, Default, Clone)]
pub struct AppData<'a> {
    pub handlebars: Handlebars<'a>,
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