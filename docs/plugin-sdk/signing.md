# T1 mod signing (draft)

Unibrowser will treat **native cdylib mods** like small packages: manifest + library + optional signature metadata. This document describes the **T1** trust tier planned for community distribution (not implemented cryptographically yet).

## Trust tiers (roadmap)

| Tier | Who | Host behaviour |
| --- | --- | --- |
| **T0** | Local dev, sideload | Load when enabled; no signature required (default). |
| **T1** | Open registry / signed releases | Publisher key signs manifest digest; host verifies before load. |
| **T2** | Curated store | T1 plus host policy (allowlist, revocation list). |

## T1 package layout (target)

```
~/.config/unibrowser/mods/my-mod/
  info.json          # ModManifest (name, version, unibrowser_api, library)
  state.json         # { "enabled": true } (host-managed)
  libmy_mod.so       # or path from info.json `library`
  ub-mod.sig         # one line, see below
```

## `ub-mod.sig` format (stub today)

Single non-empty line:

```
t1;sha256=<hex-digest-of-canonical-manifest>;sig=<base64-signature-bytes>
```

**Stub verifier (current):** checks prefix and fields only; does **not** verify Ed25519 yet. Real T1 will:

1. Canonicalise `info.json` (sorted keys, UTF-8, no BOM).
2. Hash with SHA-256; compare to `sha256=`.
3. Verify `sig=` with publisher public key from a future trust store (`trusted-publishers.json` or OS keychain).

## Host environment flags

| Variable | Effect |
| --- | --- |
| `UNIBROWSER_REQUIRE_SIGNED_MODS=1` | Refuse to load unsigned native mods (missing or invalid `ub-mod.sig`). Listing in the mod portal still shows them with a blocked reason. |
| `UNIBROWSER_LOAD_CDYLIB_MOD=1` | Dev-only: also try loading the in-tree stub `.so` paths (see README). |

## Author checklist (future T1)

1. Generate keypair (tooling TBD); register publisher id with Unibrowser trust store.
2. Sign canonical `info.json` digest; write `ub-mod.sig`.
3. Ship mod folder or `.ubmod` zip with manifest, library, and signature.
4. Pin `unibrowser_api` in `info.json` to the host `MOD_API_VERSION`.
