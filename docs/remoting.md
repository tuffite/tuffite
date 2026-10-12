# Remoting

Native Remoting client for Rust applications hosted by Tuffite. The `tuffite::remoting` module wraps
callback-based ShellAPI calls as async operations and converts native session,
stats and screenshot results. It contains no application state, authorization
policy, Link transport or Tokio runtime owner.

Construct `Remoting::new(Some(context.shell_api()))` in the application's
composition root. Refresh capabilities before starting a control session; use
the returned session ID for signaling, stats, clipboard/input and stop requests.
`RemotingStartOptions`, `RemotingSignal` and `RemotingInputEvent` are reexported
for consumers. Start outcomes that arrive after the caller has cancelled are
stopped automatically so native session capacity is not leaked.

The application owns its executor and stops sessions before shutting it down.
Actual native integration requires a Tuffite Framework with Remoting support;
pure Rust compilation and Clippy do not require a Chromium build.

Enable the public SDK with `tuffite = { version = "0.1.0", features = ["remoting"] }`.

```rust
use tuffite::remoting::Remoting;
let remoting = Remoting::new(Some(context.shell_api()));
```

Native connections, ICE sockets, WebRTC callbacks and session teardown are owned
by a dedicated Chromium IO message loop. The browser UI thread only forwards
asynchronous service requests; snapshot results return asynchronously on their
caller's sequence. Input injectors retain the actual UI task runner for platform
operations. A UI message pump must never be passed off as the WebRTC network
thread: POSIX socket descriptor watches require an IO pump.
