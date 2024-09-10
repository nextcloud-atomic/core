# Activation flow

```mermaid
sequenceDiagram
    ServerStart->>Caddy: starts container
    ServerStart->>Activation Service: starts container
    User->>Activation Service: Provides admin password
    Activation Service->>NC AiO: Generate and provide credentials
    Activation Service->>NC AiO: Wait for services to start
    Activation Service->>Caddy: Configure proxy to forward to Nextcloud
    Activation Service-->>User: Forward to NC
    Activation Service->>Activation Service: Shutdown
```