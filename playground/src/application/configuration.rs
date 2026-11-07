//! Typed runtime declarations installed by `Application::setup`.
use tuffite::configuration::{
    Action, Behavior, Configuration, Explorer, Grant, IsolatedWorld, Mode, Network, Route, Rule,
    Scope, Window,
};

pub(super) fn build() -> Configuration {
    let mut config = Configuration::new(["playground", "playground-resource"]);
    config = config.window(Window {
        label: "main".into(),
        url: "playground://main/ui/dist/index.html".into(),
        title: "Tuffite Playground".into(),
        width: 1100,
        height: 1000,
        min_width: Some(320),
        min_height: Some(240),
        max_width: Some(1920),
        max_height: Some(1200),
        ..Window::default()
    });
    config = config.window(Window {
        label: "secondary".into(),
        url: "playground://main/ui/dist/window.html".into(),
        title: "Tuffite Secondary Window".into(),
        width: 560,
        height: 420,
        focused: Some(false),
        min_width: Some(320),
        min_height: Some(240),
        max_width: Some(1920),
        max_height: Some(1200),
        ..Window::default()
    });
    config = config.route(
        Route::new()
            .origins([
                "playground://main",
                "playground://*.artifact",
                "playground://*.preview",
                "playground://public",
                "playground://business",
                "playground://behavior.fixture",
                "playground-resource://main",
            ])
            .path("/ui/dist/*")
            .directory("ui"),
    );
    config = config.route(
        Route::new()
            .origins(["playground://parser.utility"])
            .path("/*")
            .directory("ui/playground-public/utilities/parser"),
    );
    config = config.protocol(
        tuffite::configuration::Protocol::new("playground").route(
            Route::new()
                .origin("playground://main")
                .origin("playground://ui")
                .path("/framework/*")
                .path("/dialog/open")
                .resolver(),
        ),
    );
    config.app.network = Some(Network {
        max_response_bytes: Some(1_048_576),
        timeout_seconds: Some(30),
        ..Network::default()
    });
    config.browser.devtools = true;
    config = config.rule(Rule {
        origins: vec!["playground://main".into()],
        identifier: Some("app-shell".into()),
        shell_api: vec![
            Grant::from("playground.Commands.DoSomething"),
            Grant::from("playground.Commands.CheckUtilitySession"),
            Grant::from("playground.Commands.CheckJavascriptExtension"),
            Grant::from("playground.Commands.EchoMojom"),
            Grant::from("playground.Echo.RoundTrip"),
            Grant::from("playground.Echo.Restricted"),
            Grant::from("playground.Aux.RoundTrip"),
            Grant::from("playground.Aux.ReadState"),
            Grant::from("playground.Aux.ProcessId"),
            Grant::from("playground.Diagnostics.ProcessId"),
            Grant::from("playground.Profiles.*"),
            Grant::from("playground.BrowserDiagnostics.*"),
            Grant::from("tuffite.Storage.*"),
            Grant::from("tuffite.Log.*"),
            Grant::from("tuffite.Utility.*"),
            Grant {
                method: "tuffite.Fs.ReadFile".into(),
                allow: Some(vec![Scope::Path("resource/*".into())]),
                deny: vec![],
            },
            Grant {
                method: "tuffite.Fs.Exists".into(),
                allow: Some(vec![Scope::Path("resource/*".into())]),
                deny: vec![],
            },
            Grant {
                method: "tuffite.Fs.*".into(),
                allow: Some(vec![
                    Scope::Path("data/*".into()),
                    Scope::Path("cache/*".into()),
                    Scope::Path("temp/*".into()),
                ]),
                deny: vec![],
            },
            Grant {
                method: "tuffite.Path.Resolve".into(),
                allow: Some(vec![
                    Scope::Path("resource/*".into()),
                    Scope::Path("data/*".into()),
                    Scope::Path("cache/*".into()),
                    Scope::Path("temp/*".into()),
                ]),
                deny: vec![],
            },
            Grant {
                method: "tuffite.Net.Request".into(),
                allow: Some(vec![
                    Scope::Url("http://example.com/*".into()),
                    Scope::Url("https://example.com/*".into()),
                    Scope::Url("http://127.0.0.1/*".into()),
                    Scope::Url("https://127.0.0.1/*".into()),
                ]),
                deny: vec![],
            },
            Grant::from("tuffite.Path.Directories"),
            Grant::from("tuffite.Dialog.OpenFile"),
            Grant::from("tuffite.Dialog.OpenDirectory"),
            Grant::from("tuffite.Window.Current"),
            Grant::from("tuffite.Window.List"),
            Grant::from("tuffite.Window.Create"),
            Grant {
                method: "tuffite.Window.Focus".into(),
                allow: Some(vec![Scope::Window("*".into())]),
                deny: vec![],
            },
            Grant {
                method: "tuffite.Window.Minimize".into(),
                allow: Some(vec![Scope::Window("*".into())]),
                deny: vec![],
            },
            Grant {
                method: "tuffite.Window.Maximize".into(),
                allow: Some(vec![Scope::Window("*".into())]),
                deny: vec![],
            },
            Grant {
                method: "tuffite.Window.Restore".into(),
                allow: Some(vec![Scope::Window("*".into())]),
                deny: vec![],
            },
            Grant {
                method: "tuffite.Window.SetFullscreen".into(),
                allow: Some(vec![Scope::Window("*".into())]),
                deny: vec![],
            },
            Grant {
                method: "tuffite.Window.Close".into(),
                allow: Some(vec![Scope::Window("*".into())]),
                deny: vec![],
            },
            Grant {
                method: "tuffite.Window.SetTitle".into(),
                allow: Some(vec![Scope::Window("*".into())]),
                deny: vec![],
            },
            Grant {
                method: "tuffite.Window.GetBounds".into(),
                allow: Some(vec![Scope::Window("*".into())]),
                deny: vec![],
            },
            Grant {
                method: "tuffite.Window.SetBounds".into(),
                allow: Some(vec![Scope::Window("*".into())]),
                deny: vec![],
            },
        ],
        explorer: Some(Explorer {
            create: Some(true),
            execute_javascript: Some(true),
            behavior: [
                (
                    Action::Navigation,
                    Behavior {
                        mode: Mode::Delegate,
                        timeout_ms: None,
                    },
                ),
                (
                    Action::Dialog,
                    Behavior {
                        mode: Mode::Delegate,
                        timeout_ms: None,
                    },
                ),
                (
                    Action::ExternalProtocol,
                    Behavior {
                        mode: Mode::Browser,
                        timeout_ms: None,
                    },
                ),
                (
                    Action::NewWindow,
                    Behavior {
                        mode: Mode::Delegate,
                        timeout_ms: None,
                    },
                ),
            ]
            .into_iter()
            .collect(),
        }),
        ..Rule::default()
    });
    config = config.rule(Rule {
        origins: vec![
            "playground://*.artifact".into(),
            "playground://*.preview".into(),
        ],
        identifier: Some("artifact-and-preview".into()),
        shell_api: vec![
            Grant::from("tuffite.Path.Directories"),
            Grant::from("tuffite.Storage.Get"),
            Grant {
                method: "tuffite.Fs.ReadFile".into(),
                allow: Some(vec![Scope::Path("resource/ui/**".into())]),
                deny: vec![Scope::Path("resource/ui/index.html".into())],
            },
        ],
        ..Rule::default()
    });
    config = config.rule(Rule {
        origins: vec!["playground://demo.artifact".into()],
        identifier: Some("artifact-app-command".into()),
        shell_api: vec![Grant::from("playground.Commands.DoSomething")],
        ..Rule::default()
    });
    config = config.rule(Rule {
        origins: vec!["playground://business".into()],
        identifier: Some("business-only".into()),
        shell_api: vec![
            Grant::from("playground.Commands.DoSomething"),
            Grant::from("playground.Echo.RoundTrip"),
        ],
        ..Rule::default()
    });
    config = config.rule(Rule {
        origins: vec![
            "playground://main".into(),
            "playground://*.artifact".into(),
            "playground://*.preview".into(),
            "playground://public".into(),
            "playground://business".into(),
            "playground://parser.utility".into(),
            "playground://demo.artifact".into(),
        ],
        identifier: Some("document-csp".into()),
        csp: Some(
            concat!(
                "default-src 'self'; ",
                "script-src 'self' 'unsafe-inline'; ",
                "style-src 'self' 'unsafe-inline'; ",
                "connect-src 'self' http://127.0.0.1:* ws://127.0.0.1:*",
            )
            .into(),
        ),
        ..Rule::default()
    });
    config = config.rule(Rule {
        origins: vec!["playground://behavior.fixture".into()],
        csp: Some("default-src 'self'; script-src 'self'; img-src 'none'".into()),
        ..Rule::default()
    });
    config = config.rule(Rule {
        origins: vec!["playground-resource://main".into()],
        csp: Some("default-src 'self'; script-src 'self'; worker-src 'self'".into()),
        ..Rule::default()
    });
    for name in ["weather", "weatherOther"] {
        config.app.isolated_worlds.insert(
            name.into(),
            IsolatedWorld {
                directory: "ui/playground-public/isolated".into(),
                entry: "card.js".into(),
                origins: vec!["playground://main".into()],
            },
        );
    }
    config
}
