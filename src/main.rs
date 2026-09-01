use anyhow::Context;
use xdg_desktop_portal_omarchy::portals::access::Access;
use xdg_desktop_portal_omarchy::portals::account::Account;
use xdg_desktop_portal_omarchy::portals::app_chooser::AppChooser;
use xdg_desktop_portal_omarchy::portals::background::Background;
use xdg_desktop_portal_omarchy::portals::dynamic_launcher::DynamicLauncher;
use xdg_desktop_portal_omarchy::portals::email::Email;
use xdg_desktop_portal_omarchy::portals::file_chooser::FileChooser;
use xdg_desktop_portal_omarchy::portals::inhibit::Inhibit;
use xdg_desktop_portal_omarchy::portals::lockdown::Lockdown;
use xdg_desktop_portal_omarchy::portals::notification::Notification;
use xdg_desktop_portal_omarchy::portals::screenshot::Screenshot;
use xdg_desktop_portal_omarchy::portals::settings::{self, Settings};
use xdg_desktop_portal_omarchy::portals::wallpaper::Wallpaper;
use xdg_desktop_portal_omarchy::portals::PortalCtx;
use xdg_desktop_portal_omarchy::{APP_ID, DBUS_NAME, DBUS_PATH};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "--help" || a == "-h") {
        println!(
            "xdg-desktop-portal-omarchy\n  Omarchy backend for xdg-desktop-portal\n\n  --demo file-chooser   Open the file picker without D-Bus\n  --demo app-chooser    Open the app chooser without D-Bus\n  --demo access         Open the access dialog without D-Bus\n  --demo account        Open the account dialog without D-Bus\n"
        );
        return Ok(());
    }
    if args.first().map(|s| s.as_str()) == Some("--picker") {
        return xdg_desktop_portal_omarchy::picker::child_main();
    }
    if args.first().map(|s| s.as_str()) == Some("--demo") {
        return demo(args.get(1).map(|s| s.as_str()).unwrap_or("file-chooser"));
    }

    tracing::info!("starting {APP_ID} on {DBUS_NAME}");

    let connection = zbus::connection::Builder::session()?
        .build()
        .await
        .context("connect to session bus")?;

    let ctx = PortalCtx::new(connection.clone());
    let server = connection.object_server();
    server.at(DBUS_PATH, FileChooser(ctx.clone())).await?;
    server.at(DBUS_PATH, Settings::load()).await?;
    server.at(DBUS_PATH, AppChooser(ctx.clone())).await?;
    server.at(DBUS_PATH, Account(ctx.clone())).await?;
    server.at(DBUS_PATH, Access(ctx.clone())).await?;
    server
        .at(DBUS_PATH, Notification::new(ctx.clone()))
        .await?;
    let inhibit = Inhibit::new(ctx.clone());
    inhibit.spawn_watch(connection.clone());
    server.at(DBUS_PATH, inhibit).await?;
    server.at(DBUS_PATH, Email(ctx.clone())).await?;
    server.at(DBUS_PATH, Wallpaper(ctx.clone())).await?;
    server.at(DBUS_PATH, Lockdown::default()).await?;
    server.at(DBUS_PATH, Screenshot(ctx.clone())).await?;
    server.at(DBUS_PATH, Background(ctx.clone())).await?;
    server.at(DBUS_PATH, DynamicLauncher(ctx.clone())).await?;
    connection.request_name(DBUS_NAME).await?;

    tokio::spawn(settings::watch_theme(connection.clone()));
    tracing::info!("portal interfaces exported at {DBUS_PATH}");

    std::future::pending::<()>().await;
    Ok(())
}

fn demo(kind: &str) -> anyhow::Result<()> {
    let token = tokio_util::sync::CancellationToken::new();
    match kind {
        "file-chooser" => {
            let req = xdg_desktop_portal_omarchy::ui::FileChooserRequest {
                title: "Open File".into(),
                accept_label: "Open".into(),
                mode: xdg_desktop_portal_omarchy::ui::FileMode::Open,
                multiple: false,
                directory: false,
                filters: vec![],
                current_filter: None,
                choices: vec![],
                current_folder: xdg_desktop_portal_omarchy::paths::home_dir(),
                current_name: String::new(),
                save_names: vec![],
            };
            let out = xdg_desktop_portal_omarchy::ui::run_file_chooser(req, token);
            println!("{out:?}");
        }
        "access" => {
            let req = xdg_desktop_portal_omarchy::ui::AccessRequest {
                title: "Allow access?".into(),
                subtitle: "xdg-desktop-portal-omarchy demo".into(),
                body: "This is a demo of the Access portal.".into(),
                deny_label: "Deny".into(),
                grant_label: "Allow".into(),
                icon: Some("dialog-password".into()),
                choices: vec![
                    xdg_desktop_portal_omarchy::dict::Choice {
                        id: "remember".into(),
                        label: "Remember this decision".into(),
                        options: vec![],
                        selected: "false".into(),
                    },
                    xdg_desktop_portal_omarchy::dict::Choice {
                        id: "scope".into(),
                        label: "Access scope".into(),
                        options: vec![
                            ("read".into(), "Read only".into()),
                            ("write".into(), "Read and write".into()),
                        ],
                        selected: "read".into(),
                    },
                ],
            };
            println!("{:?}", xdg_desktop_portal_omarchy::ui::run_access(req, token));
        }
        "app-chooser" => {
            let req = xdg_desktop_portal_omarchy::ui::AppChooserRequest {
                title: "Open with".into(),
                choices: vec![],
                last_choice: None,
                content_type: Some("text/plain".into()),
                uri: Some("file:///tmp/demo.txt".into()),
                filename: Some("demo.txt".into()),
            };
            println!("{:?}", xdg_desktop_portal_omarchy::ui::run_app_chooser(req, token));
        }
        "account" => {
            let username = xdg_desktop_portal_omarchy::paths::whoami();
            let real_name = xdg_desktop_portal_omarchy::paths::real_name();
            let image = xdg_desktop_portal_omarchy::paths::account_image(None);
            let req = xdg_desktop_portal_omarchy::ui::AccountRequest {
                title: "Share user info with this application?".into(),
                subtitle: "The application will be able to see your username, full name, and profile picture.\n\nReason: “Omarchy portal demo.”".into(),
                username,
                real_name,
                image: Some(image.to_string_lossy().into_owned()),
            };
            println!("{:?}", xdg_desktop_portal_omarchy::ui::run_account(req, token));
        }
        other => anyhow::bail!("unknown demo {other}"),
    }
    Ok(())
}
