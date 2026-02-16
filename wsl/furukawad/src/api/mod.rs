pub mod create;
pub mod list;
pub mod start;
pub mod stop;
pub mod delete;
pub mod logs;
pub mod inspect;
pub mod version;
pub mod info;
pub mod images;
pub mod middleware;

use axum::{routing::{get, post}, Router};
use crate::state::AppState;

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/version", get(version::handle))
        .route("/info", get(info::handle))
        .route("/images/json", get(images::list::handle))
        .route("/images/create", post(images::create::handle))
        .route("/containers/create", post(create::handle))
        .route("/containers/json", get(list::handle))
        .route("/containers/:id/start", post(start::handle))
        .route("/containers/:id/stop", post(stop::handle))
        .route("/containers/:id/logs", get(logs::handle))
        .route("/containers/:id/json", get(inspect::handle))PS C:\Projects\rustckere furukawaedition> cmd
Microsoft Windows [Version 10.0.26200.7840]
(c) Microsoft Corporation. All rights reserved.

C:\Projects\rustckere furukawaedition>Z: 

Z:\>cd desktop

Z:\desktop>npm run tauri dev

> desktop@0.0.0 tauri
> tauri dev

     Running BeforeDevCommand (`npm run dev`)

> desktop@0.0.0 dev
> vite

17:06:17 [vite] (client) Re-optimizing dependencies because vite config has changed

  VITE v7.3.1  ready in 773 ms

  ➜  Local:   http://localhost:5173/
  ➜  Network: use --host to expose
     Running DevCommand (`cargo  run --no-default-features --color always --`)
        Info Watching Z:\desktop\src-tauri for changes...
17:06:29 [vite] (client) Pre-transform error: Failed to load url /src/main.tsx (resolved id: C:/Projects/rustckere furukawaedition/desktop/src/main.tsx). Does the file exist?
17:06:47 [vite] (client) Pre-transform error: Failed to load url /src/main.tsx (resolved id: C:/Projects/rustckere furukawaedition/desktop/src/main.tsx). Does the file exist? (x2)
   Compiling proc-macro2 v1.0.*｝+))
        .layer(axum::middleware::from_fn(middleware::trace_request))
        .with_state(state)
}

