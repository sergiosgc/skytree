use actix_web::{web, App, HttpServer};
use handlebars::Handlebars;
use skytree::{skytree::{host_group::HostGroup, rest::RestCollection}, AppData, Config};
use clap::{Parser, command};

#[derive(Parser, Debug)]
#[command(name = "skytree")]
#[command(author = "Sérgio Carvalho")]
#[command(version = "1.0")]
#[command(about = "Web and REST API management of Ansible Inventory", long_about = None)]
struct CliArguments {
    #[arg(short='c', long="config-file")]
    config_file: Option<String>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let mut cli = CliArguments::parse();
    if cli.config_file.is_none() {
        cli.config_file = Some("/etc/skytree.ini".to_string());
    }
    {
        let ini_file_settings = ini::ini!(&cli.config_file.unwrap());
        let mut config_template_dir = "templates".to_string();
        if let Some(settings) = ini_file_settings.get("skytree") {
            config_template_dir = settings.get("template_dir").unwrap_or(&Some(config_template_dir.clone())).as_deref().unwrap_or(&config_template_dir).to_string();
        }
        if config_template_dir.len() == 0 {
            config_template_dir = "templates".to_string();
        }
        if config_template_dir.chars().next().unwrap() != '/' {
            config_template_dir = format!("{}/{}", std::env::current_dir().unwrap().to_str().unwrap().to_string(), config_template_dir);
        }
        if config_template_dir.chars().rev().next().unwrap() != '/' {
            config_template_dir = format!("{}/", config_template_dir);
        }
        unsafe {
            skytree::CONFIG.template_dir = Box::leak(config_template_dir.into_boxed_str());
        }
    }
    let mut handlebars = Handlebars::new();
    handlebars.set_strict_mode(true);
    handlebars.register_templates_directory(".hbs", Config::get_template_dir()).unwrap();
    let app_data = web::Data::new( AppData { handlebars });

    HttpServer::new(move || {
        App::new()
            .app_data(app_data.clone())
            .route("/host_groups", web::get().to(HostGroup::get))
            .route("/host_groups/{dummy}/some/{variable}", web::get().to(HostGroup::get))
    })
    .bind(("127.0.0.1", 3000))?
    .run()
    .await
}
