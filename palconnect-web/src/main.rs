
use std::path::Path;

use actix_files::Files;
use actix_web::{App, HttpServer};
use leptos::prelude::*;
use leptos_actix::{LeptosRoutes, generate_route_list};
use leptos_meta::*;
use dotenv::dotenv;
use surrealdb::engine::remote::ws::Ws;
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;

use palconnect_web::{app::App as PalconnectWebApp, components::*, views::*};


#[actix_web::main]
async fn main() -> std::io::Result<()> {

    dotenv().ok();

    // Load the Leptos configuration from the Cargo.toml file.
    let conf = get_configuration(Some(&format!("{}/Cargo.toml", env!("CARGO_MANIFEST_DIR"))))
        .expect("failed to load Leptos configuration");
    let addr = conf.leptos_options.site_addr;

    // Set up Actix web server and Leptos app
    HttpServer::new(move || {
        // Let's grab the generated routes for our Leptos app. Catch options, head, methods.
        let routes = generate_route_list(PalconnectWebApp);
        let leptos_options = &conf.leptos_options;
        let pkg_dir = Path::new(&*leptos_options.site_root).join(&*leptos_options.site_pkg_dir);
        let pkg_route = format!("/{}", leptos_options.site_pkg_dir.trim_matches('/'));

        // Build app.
        App::new()
            .leptos_routes(routes, {
                let leptos_options = leptos_options.clone();
                move || {
                    view! {

                      // This is the root of the HTML document view.
                      <!DOCTYPE html>
                      <html lang="en">
                        <head>
                          <meta charset="utf-8"/>
                          <meta name="viewport" content="width=device-width, initial-scale=1"/>
                          <Stylesheet id="leptos" href=leptos_options.css_path() />
                          <HydrationScripts options=leptos_options.clone() />
                          <MetaTags />
                        </head>
                        <body>

                          // * All of our Leptos views will be added inside this <body> tag. The <PalconnectWebApp/> component is the root of our app, and it will render all of the other components and views that make up our app.

                          <PalconnectWebApp />

                          // ? Space here for any additional scripts that need to be added to the <body> tag. For example, if you want to add a script that runs after the page has loaded, you can add it here.

                        </body>
                      </html>

                    }
                }
            })
            .service(Files::new(&pkg_route, pkg_dir))
    })
    .bind(&addr)?
    .run()
    .await
}
