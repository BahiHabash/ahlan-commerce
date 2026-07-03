# GraphQL Client Choice

## Why TanStack Query + Thin Client?

We have chosen React with **TanStack Router** and **TanStack Query** alongside a deliberately thin GraphQL client wrapper. 

### Rationale

This combination teaches the specific "admin UI split" that Majaz expects:
- **TanStack Router:** Owns the URL and screen state.
- **TanStack Query:** Owns the server state, caching, loading, error states, and refetching logic.
- **GraphQL Client (`graphql-request`):** Owns the transport layer and GraphQL protocol behavior.

We avoid hiding server state in React component state (`useState`, `useEffect`). Component state should be strictly reserved for local UI concerns (e.g., form input fields, toggling UI elements). Server state inherently involves cache keys, invalidation mechanisms, loading/error states, and retry behaviors that are fundamentally different from local UI state.

### Why Apollo Client is a Mentor Extension

Apollo Client is a robust ecosystem that includes both the transport layer and an advanced normalized cache. While powerful, it obscures the distinct problems that TanStack Query and a generic transport client solve individually. By using TanStack Query, the boundaries between fetching, caching, and routing are explicit, which is an intentional architectural learning goal for Ahlan. Apollo Client is therefore considered a mentor extension, rather than the default Ahlan implementation.
