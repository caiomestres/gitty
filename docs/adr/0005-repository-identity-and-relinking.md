# Repository identity by root-commit fingerprint with collision-safe re-linking

Each Repository is identified by a Gitty-assigned UUID persisted in Config. To preserve that identity across filesystem moves, Gitty fingerprints a Repository by its root-commit object ID (the OID of the initial commit). On rescan, a registered Repository whose recorded path has vanished is marked Missing rather than deleted.

Re-linking is deliberately conservative because the root-commit fingerprint is **not** globally unique — clones and forks of the same upstream share it, and a freshly-initialised repository has no commit at all (null fingerprint). Gitty therefore auto-relinks only when the match is unambiguous: exactly one Missing Repository and exactly one newly-discovered repository share a single non-null fingerprint. Ambiguous matches (a fingerprint shared by multiple candidates) and null fingerprints are never auto-linked — the Missing Repository stays Missing and the new one is registered fresh.

There is no purely content-based way to distinguish two clones of the same repository (the path is the only differentiator, and the path is exactly what changed), so guessing is avoided. Manual resolution of ambiguous matches is deferred.
