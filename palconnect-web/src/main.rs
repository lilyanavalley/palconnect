use actix_web::{App, HttpServer};
use leptos::prelude::*;
use leptos_actix::{generate_route_list, LeptosRoutes};
use leptos_meta::MetaTags;
use palconnect_web::app::App as PalconnectWebApp;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let conf = get_configuration(None).expect("failed to load Leptos configuration");
    let addr = conf.leptos_options.site_addr;

    HttpServer::new(move || {
        let routes = generate_route_list(PalconnectWebApp);
        let leptos_options = &conf.leptos_options;

        App::new().leptos_routes(routes, {
            let leptos_options = leptos_options.clone();
            move || {
                view! {
                    <!DOCTYPE html>
                    <html lang="en">
                        <head>
                            <meta charset="utf-8"/>
                            <meta name="viewport" content="width=device-width, initial-scale=1"/>
                            <HydrationScripts options=leptos_options.clone() />
                            <MetaTags />
                        </head>
                        <body>
                            <PalconnectWebApp />
                        </body>
                    </html>
                }
            }
        })
    })
    .bind(&addr)?
    .run()
    .await
}
