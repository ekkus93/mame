# DG-7 External Artwork Provider Policy

**Date:** 2026-09-12  
**Repository:** `ekkus93/mame`  
**Task:** MT-804 — Define external artwork-provider policy  
**Decision gate:** DG-7 — network-backed artwork providers

## Decision

**NO-GO for enabling a network-backed artwork provider by default at this stage.**

The application may add a specific external artwork provider only after that provider has been explicitly nominated, its API and content terms have been reviewed, and its adapter satisfies the licensing, attribution, privacy, caching, offline, and rate-limit requirements in this policy. Local artwork remains the default and fully supported artwork path.

This is a policy gate, not a prohibition on future provider work. It deliberately prevents an implementation from silently turning artwork display into an unbounded network, privacy, licensing, or availability dependency.

## Licensing

A provider adapter must document, before merge:

- the provider/API terms governing automated access;
- the license or other permitted-use basis for the returned artwork itself;
- whether local caching is allowed and for how long;
- whether redistribution, export, or backup of cached artwork is allowed;
- any restrictions that differ between metadata returned by the API and the underlying image/content rights.

Adapters must not scrape a service that prohibits automated access, bypass access controls, or infer that API access grants rights to redistribute third-party images. If the provider's terms do not clearly permit an intended operation, that operation remains disabled.

## Attribution and provenance

Every network-sourced artwork record must retain enough provenance to satisfy the provider and content-license requirements. Where applicable that includes:

- provider identity;
- source or canonical artwork URL/identifier;
- creator or contributor attribution;
- license identifier or license URL;
- required attribution text;
- retrieval timestamp and provider-specific revision/validator metadata.

The UI must display required attribution in an accessible location associated with the artwork. Cache compaction, migration, or deduplication must not strip attribution or provenance that is required to use the cached asset lawfully.

Provider-originated artwork must remain distinguishable from local user-configured artwork. A provider must never overwrite or masquerade as a local artwork source.

## Privacy and network disclosure

Network-backed artwork is **opt-in**. Enabling a provider must be an explicit user action unless a later product decision revisits DG-7 with a documented privacy review.

A provider request may disclose only the bounded lookup data required by that provider, such as a MAME short name or software-list identifier. It must not transmit:

- host filesystem paths;
- ROM, CHD, software, save-state, or application-data paths;
- ROM or disk contents;
- local artwork contents;
- arbitrary machine metadata unrelated to the lookup;
- stable device identifiers, analytics identifiers, or user account data not required for the provider operation.

Hashes or other content-derived identifiers are not implicitly approved by this policy. If a future provider requires them, that is a new privacy decision and must be documented before implementation.

Provider traffic must use HTTPS and follow the application's normal error-reporting rules. Failures must not leak secrets or local paths into logs or user-facing diagnostics.

## Caching

Network artwork caching must be bounded and provider-aware:

- use a documented disk quota and deterministic eviction policy such as LRU;
- keep network-provider cache storage separate from user-managed local artwork roots;
- retain provenance and attribution metadata with cached assets;
- honor provider terms and HTTP validators/directives such as `ETag`, `Last-Modified`, `Cache-Control`, and `Retry-After` where applicable;
- use a documented TTL when the provider does not supply usable freshness semantics;
- do not cache assets when the provider's terms prohibit caching;
- revalidate rather than redownload when supported;
- provide a way to clear provider cache without deleting local artwork.

Frontend preview caching remains independently bounded. A provider adapter must not turn a bounded WebView preview cache into a catalog-wide asset cache.

## Offline behavior and failure isolation

The library, machine-detail view, launch flow, and local artwork path must remain functional with no network connection and with every external provider unavailable.

When offline:

- permitted cached provider artwork may be shown with its retained attribution and provenance;
- an uncached provider asset falls back to the normal missing-art state;
- the UI may identify that a provider is offline/unavailable, but must not block navigation or launch;
- the application must not spin in an aggressive retry loop.

A provider outage, malformed response, authentication failure, or cache failure is an artwork-provider failure only. It must not make the catalog or emulator session unavailable.

## Rate limiting and request bounds

Every provider adapter must implement explicit request bounds appropriate to that provider:

- bounded concurrent requests;
- a provider-specific request-rate limit or token bucket;
- handling for HTTP 429 and `Retry-After`;
- exponential backoff with a bounded maximum delay for transient errors;
- a cooldown/circuit-breaker behavior after repeated failures where useful;
- cancellation or stale-result suppression when the user changes selection;
- no catalog-wide artwork prefetch by default.

Interactive lookup should be scoped to selected or otherwise immediately visible content. A provider adapter must not issue one request per catalog row merely because a large catalog has been loaded.

## Adapter acceptance requirements

A specific provider may pass DG-7 only when its implementation includes tests or repeatable qualification evidence for:

1. terms/licensing and attribution behavior;
2. opt-in configuration and bounded disclosure;
3. malformed/oversized provider responses;
4. cache quota and eviction;
5. offline fallback and provider failure isolation;
6. 429/`Retry-After` and transient-error backoff;
7. request concurrency/rate bounds;
8. cancellation/stale-response handling;
9. confirmation that local artwork and normal library interaction remain independent of provider availability.

Provider-specific credentials, if ever required, must use an appropriate secret-storage design and must not be written into general settings, logs, URLs, or repository content.

## DG-7 conclusion

**Policy defined; provider enablement remains NO-GO until a concrete provider is reviewed against it.**

MT-804 is therefore complete as a policy/decision-gate task. A later provider-specific task must name the provider, record the licensing/privacy review, implement the bounded adapter, and present qualification evidence before network-backed artwork can be enabled.
