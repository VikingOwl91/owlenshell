# ADR-0002: Atomic Surfaces as a Core Primitive

- Status: Accepted

## Context

Traditional shell/toolkit approaches can model a small visible popup inside a much larger transparent layer surface, causing invisible interaction geometry.

Owlenshell aims to treat independent shell UI as independent desktop objects.

## Decision

Make atomic surfaces a core runtime primitive.

An atomic surface has native geometry corresponding to its intended interaction region and owns independent placement/lifecycle policy.

## Consequences

Positive:

- no unnecessary invisible interaction geometry
- independent movement and persistence
- reusable topology-aware placement
- same primitive can support popups, OSDs, notifications and floating widgets

Negative:

- more native surface lifecycle management
- cross-output migration may require recreation
- input/focus behavior must be designed explicitly
