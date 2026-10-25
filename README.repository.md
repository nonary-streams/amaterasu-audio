# Standalone Amaterasu audio module source proof

Extracted from Amaterasu commit `a6b0db9e36a1a59328ab4b4785b27b210f8182b7`. The original module README is
`README.md`. This local Git repository builds one `amaterasu-audio` cdylib and
rlib, with the existing ABI/media/SDK pinned to foundation commit
`51ef23b1923335e0bb0c6af9eef685c6f4807520` by local Git URL. No public remote is configured.

The `file://` dependency is deliberately machine-local for this proof. A stable
approved Git location or registry release must replace it before collaborators
can reproduce the build elsewhere. The signed host package still controls the
module identity, artifact bytes, load order, policy and native ABI negotiation.
Extracting this source does not authorize activation or change a live package.
