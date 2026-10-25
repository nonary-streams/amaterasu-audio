# Standalone Amaterasu audio source (publication candidate)

This crate builds the `amaterasu-audio` native module. See `README.md` for its
control contract and platform limits. `Cargo.toml` pins the proposed
`nonary-streams/amaterasu-sdk` Git identity and exact SDK commit; the committed
`Cargo.lock` records the locally validated dependency resolution. Build with
`cargo test --locked --offline --no-run` after dependencies are cached. Tests
that contact a live Pulse server are separate qualification and were not run
for this preparation. The source began at Amaterasu
`a6b0db9e36a1a59328ab4b4785b27b210f8182b7` and was checked against the
latest isolated integration. No audio device, host grant or installed service
was changed. This is not a published release; the source license is pending.
