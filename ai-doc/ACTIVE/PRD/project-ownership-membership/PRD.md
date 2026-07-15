---
id: prd-project-ownership-membership
version: "0.3"
author: Sno.ai
date: 2026-07-14
status: draft
mode: technical
requires-clarification: true
---

# PRD: Project Ownership and Membership

> Define the durable ownership, collaboration, and authorization boundary for every SNO Project and every Project-scoped memory.

## 1. Executive Summary

A SNO Project is a durable collaboration scope analogous to a hosted Git repository. It is not a cloud deployment, local directory, machine, or transient CLI context. A Project exists independently of where its members work and may be bound to zero or more local directories or Stations.

Every Project has exactly one owner namespace: either an Account or an Organization. Accounts may join Projects directly. Organization-owned Projects may additionally grant access through Organization membership. Ownership may transfer between eligible Accounts and Organizations without changing the Project's immutable identity or Project-scoped memories.

All SNO memories belong to exactly one Project. Authorization to read or mutate a memory is derived from effective Project permission. Local files and CLI bindings never grant Project access by themselves.

### Success Criteria

- Every Project has one immutable `project_id` and exactly one current owner namespace.
- Every persisted memory has one valid `project_id`.
- Every Project read or mutation is authorized from server-side Account, Organization, and membership state.
- Removing an Account's final access grant immediately denies future Project and memory access.
- Ownership transfer preserves `project_id`, memories, membership records, and audit history.
- CLI commands resolve the same effective permission as the service API.

## 2. Problem Statement

SNO needs a shared unit for memory, collaboration, and identity that matches a concept users already understand. Treating Project as a local folder would prevent the same Project from spanning multiple machines. Treating it as a cloud deployment would incorrectly couple memory and collaboration to infrastructure. Treating it as a global CLI context would make access depend on mutable local state.

The intended model is repository-like:

```text
Account or Organization ──owns──▶ Project
Direct Accounts ──────────join──▶ Project
Organization grants ─────access─▶ Project
                                  ├── memories
                                  ├── Project metadata and managed files
                                  └── zero or more local bindings
```

Git itself does not define accounts or repository ownership. This PRD follows the hosted-repository ownership model: a repository-like resource belongs to one personal or organizational namespace, while other Accounts receive direct or inherited access.

Without one authoritative contract, CLI, API, memory services, and future user interfaces could independently invent incompatible concepts such as co-owners, local ownership, implicit folder access, or Organization membership that bypasses Project authorization.

## 3. Semantic Model

### 3.1 Account

An Account represents one authenticated human or automation identity. An Account may:

- own personal Projects;
- belong to Organizations;
- receive direct Project membership;
- accept Project invitations or ownership transfers;
- authenticate interactive CLI and non-interactive automation sessions.

Authentication credentials prove an Account identity. They do not grant Project access without an ownership or membership relationship.

### 3.2 Organization

An Organization is a shared owner namespace and membership boundary. An Organization may own Projects. Organization roles determine who may administer the Organization and act on its owned Projects.

Version 1 uses Organization membership directly. Team-based Project grants are deferred until a separate requirement defines team lifecycle and precedence.

### 3.3 Project

A Project is the canonical scope for shared SNO memories and Project-managed metadata/files.

Required identity fields:

- `project_id`: immutable, globally unique identifier;
- `slug`: mutable human-readable name unique within the owner namespace;
- `owner_type`: exactly `account` or `organization`;
- `owner_id`: identifier of the current owner namespace;
- `status`: `active` or `archived` in version 1;
- creation and update timestamps.

The canonical human reference is `<owner-slug>/<project-slug>`. The canonical reference may change after rename or transfer. `project_id` must not change.

### 3.4 Local Binding

A local binding associates a directory with `project_id`. It is analogous to a local checkout knowing which hosted repository it belongs to, but SNO does not implement Git content synchronization.

A binding:

- helps the CLI resolve the current Project by walking from the working directory toward its filesystem root;
- may exist on multiple machines and in multiple directories;
- contains no authorization grant;
- must not include Account tokens or Project secrets;
- may be removed without deleting or leaving the Project.

### 3.5 Project-Scoped Memory

Every persisted SNO memory must include `project_id`. A memory without a valid Project scope is rejected. Account-global, Organization-global, and unscoped memories are outside this PRD and require separate explicit contracts.

Project transfer, rename, local unlink, member removal, and Station replacement must not rewrite memory ownership. Memory authorization follows current Project permission at request time.

## 4. Actor Registry

### Actor: Personal Project Owner

- **Role**: Account that owns a personal Project.
- **Motivation**: Establish a durable memory scope and collaborate without creating an Organization.
- **Entry points**: CLI, API, future user interface.
- **Success predicates**:
  - can create, rename, archive, and transfer the Project;
  - can invite and remove direct members;
  - cannot leave while remaining the owner;
  - ownership-sensitive mutations require fresh authorization and audit records.

### Actor: Organization Owner or Administrator

- **Role**: Account authorized to administer an Organization and its Projects.
- **Motivation**: Keep team Projects owned by the team rather than one employee.
- **Entry points**: CLI, API, future Organization administration interface.
- **Success predicates**:
  - can create or accept transfer of an Organization-owned Project when Organization policy allows;
  - can manage Project access within the Organization role boundary;
  - loss of the Organization role removes inherited administrative power.

### Actor: Project Editor

- **Role**: Account that reads and writes Project memories and managed files without administrative permission.
- **Motivation**: Share the same Project knowledge scope across machines and collaborators.
- **Success predicates**:
  - can read and write Project-scoped data;
  - cannot manage members, Project settings, archives, or ownership;
  - can leave a Project when access is not solely inherited and the Account is not the owner;
  - loses access immediately after removal, departure, or loss of the granting Organization relationship.

### Actor: Project Read-Only Member

- **Role**: Account with read-only access.
- **Motivation**: Inspect Project knowledge without mutating it.
- **Success predicates**:
  - can read visible Project metadata and memories;
  - every Project, memory, membership, and managed-file mutation is denied.

### Actor: CLI or AI Agent

- **Role**: Machine actor operating with an Account credential and optional local Project binding.
- **Motivation**: Resolve Project scope deterministically and perform authorized work non-interactively.
- **Success predicates**:
  - current-directory resolution produces one `project_id` or an explicit error;
  - `--project` may select an accessible Project for one command without changing the local binding;
  - JSON output is deterministic and contains no credentials.

### Actor: Authorization Service

- **Role**: Server-side policy enforcement point for Project and memory operations.
- **Motivation**: Produce one deterministic permission result shared by all clients.
- **Success predicates**:
  - every protected operation resolves effective permission from authoritative records;
  - stale local bindings, cached membership, and client-supplied roles cannot grant access;
  - decisions and membership mutations are auditable without logging credentials or private memory content.

## 5. Ownership Model

### 5.1 Single Owner Namespace

A Project has exactly one owner namespace at a time:

```text
owner = Account(account_id) | Organization(organization_id)
```

Co-ownership is forbidden. Multiple Accounts may have administrative permission, but they do not become additional owner namespaces.

### 5.2 Personal Ownership

For an Account-owned Project:

- the owner Account has full owner permission;
- other Accounts receive direct Project roles;
- Organization membership does not implicitly grant access;
- the Project may transfer to another Account or an Organization.

### 5.3 Organization Ownership

For an Organization-owned Project:

- the Organization is the owner namespace;
- Organization owners receive Project owner-level administration through the Organization relationship;
- Organization administrators receive the Project permission defined by Organization policy;
- ordinary Organization members receive no Project access unless the Project grants an inherited Organization role;
- direct outside members may be invited when Organization policy allows;
- the Project may transfer to an Account or another Organization when both source and target policies allow.

### 5.4 Ownership Transfer

Transfer changes `owner_type`, `owner_id`, and potentially the canonical `<owner>/<project>` reference. It must preserve:

- `project_id`;
- memories and Project-managed data;
- direct membership unless target policy rejects a grant;
- audit history;
- active local bindings, which continue to reference `project_id` rather than the mutable owner/slug path.

Account-to-Account transfer requires acceptance by the target Account. Transfer to an Organization requires the initiating Account to have permission to create or accept Projects in the target Organization. Transfer is not complete until all target policy checks succeed atomically.

## 6. Permission Model

### 6.1 Project Roles

Version 1 defines exactly three effective Project roles. Owner includes all administrative permission; there is no separate administrator or maintainer role.

| Role | Read Project data | Write memories/files | Manage members | Rename/settings | Archive | Transfer ownership |
|---|---:|---:|---:|---:|---:|---:|
| Read-only | Yes | No | No | No | No | No |
| Editor | Yes | Yes | No | No | No | No |
| Owner | Yes | Yes | Yes | Yes | Yes | Yes |

An Organization policy may further restrict an action. It must not elevate an Account above the effective Project role defined here.

### 6.2 Grant Sources

An Account may receive Project permission from:

1. personal ownership;
2. Organization ownership administration;
3. direct Project membership;
4. inherited Organization Project access.

The effective role is the highest valid role across current grants, subject to Organization restrictions. Grant evaluation must be deterministic and server-side.

### 6.3 Deny and Revocation Rules

- Missing or expired authentication denies access.
- No valid ownership or membership grant denies access.
- Archived Projects are read-only except for owner-authorized restoration or final disposition.
- Removing a direct grant does not remove independently inherited Organization access.
- Removing Organization membership removes all access inherited only from that Organization.
- Local bindings and previously downloaded data do not restore server access.
- The owner cannot leave; ownership must transfer or the Project must be archived first.

## 7. Membership Lifecycle

### 7.1 Invitation

An owner may invite an Account as an editor or read-only member. An invitation records:

- Project ID;
- invited Account or verified invitation address;
- proposed role;
- inviter Account;
- created time and expiry;
- accepted, revoked, expired, or pending status.

Invitations may target either an existing Account identifier or a verified email address. An email invitation may remain pending before the recipient creates an Account; acceptance binds it to the authenticated Account that proves control of that address. The expiry duration remains a product-policy decision.

### 7.2 Join

An invited Account joins by accepting the invitation while authenticated as its intended recipient. Acceptance atomically creates or updates the direct membership and consumes the invitation.

An Account must not join a private Project by knowing its ID or canonical name. Public discovery and public Project access are outside this PRD.

### 7.3 Role Change

Role changes require owner permission and must not:

- assign the owner role through a membership mutation;
- create a second owner namespace;
- demote the last effective Organization owner through a Project operation;
- bypass Organization policy.

### 7.4 Remove and Leave

- Owners may remove direct members.
- A direct member may leave voluntarily.
- An Account whose access is only inherited from an Organization cannot leave the Project independently; it must leave the Organization or an Organization administrator must change the inherited grant.
- Removal or departure preserves Project data and audit records.
- The removed Account immediately loses new server access.

## 8. CLI Contract

The ownership PRD defines command semantics. The unified CLI PRD owns parsing, output conventions, package distribution, and external subcommands.

### 8.1 Account and Organization Context

```text
sno account whoami
sno org list
sno org show <org>
```

Project discovery has one canonical entry point: `sno project list [--owner <account-or-org>]`. Account and Organization command groups must not add aliases for the same operation. Organization creation, billing, teams, and broad member administration require a separate Organization PRD. This document only defines the Organization relationship needed for Project ownership and inherited access.

### 8.2 Project Identity and Local Binding

```text
sno project init [name] [--owner <account-or-org>]
sno project link <owner/project>
sno project unlink
sno project status
sno project show [owner/project]
sno project list [--owner <account-or-org>]
```

`sno project status` reports the immutable Project ID, canonical owner/name, local binding root, current Account, and effective role. It must not expose tokens.

### 8.3 Membership and Ownership

```text
sno project member list [owner/project]
sno project member invite <account> [--role editor|read-only]
sno project member role <account> <editor|read-only>
sno project member remove <account>
sno project join <invitation>
sno project leave
sno project transfer <account-or-org>
sno project archive
```

All commands support the unified CLI's JSON mode. Destructive and ownership mutations require explicit confirmation in interactive mode and an explicit non-interactive confirmation flag in automation. The exact global flag spelling is owned by the unified CLI PRD.

### 8.4 Project Resolution

Resolution precedence:

1. explicit one-command `--project <owner/project-or-id>`;
2. local binding found by walking from the current directory to the filesystem root;
3. otherwise fail with a usage error that explains how to initialize or link a Project.

There is no sticky global Project context. The current directory behaves like a Git working tree: it determines the default Project without redefining the Project itself.

## 9. Actor Flows

### Flow A: Create a Personal Project

1. Authenticated Account runs `sno project init my-project`.
2. Service creates one Project owned by that Account.
3. CLI writes a non-secret local binding containing `project_id`.
4. `sno project status` reports the Account as owner.
5. New memories created in the working tree use that `project_id`.

Expected invariant: deleting the local binding does not delete the Project or memories.

### Flow B: Invite and Join

1. Owner runs `sno project member invite member@example.com --role editor`.
2. Service creates one pending, expiring invitation.
3. Intended Account accepts through `sno project join <invitation>`.
4. Service creates a direct member grant and consumes the invitation.
5. The new member links a local directory and accesses Project memories.

Expected invariant: another Account cannot redeem the invitation.

### Flow C: Organization-Owned Project

1. Authorized Organization actor initializes or receives a Project under the Organization namespace.
2. Organization policy defines which Organization roles receive inherited Project access.
3. Direct outside members may be invited only when policy allows.
4. Authorization service combines valid direct and inherited grants.

Expected invariant: Organization membership alone grants only the role explicitly configured for that Organization-owned Project.

### Flow D: Transfer Ownership

1. Current owner requests transfer to an Account or Organization.
2. Service validates current owner authority and target eligibility.
3. Account targets accept; Organization targets pass target policy authorization.
4. Service atomically changes owner namespace.
5. Project ID, memories, audit history, and local bindings remain valid.

Expected invariant: no intermediate state has zero or two owner namespaces.

### Flow E: Remove Access

1. Authorized actor removes a direct member or the Account loses its granting Organization membership.
2. Authorization state changes atomically.
3. New Project and memory requests from that Account fail closed.
4. Project data remains owned by the Project.

Expected invariant: cached CLI state never overrides server revocation.

## 10. Constraints and Boundaries

### Authorization

- Server-side authorization is authoritative.
- All Project and memory requests use immutable IDs internally.
- Clients must not submit an effective role as trusted input.
- Membership and ownership mutations require optimistic concurrency or equivalent atomic conflict detection.
- Authorization failures do not reveal private Project existence beyond the authenticated Account's permitted view.

### Auditability

The system records actor, action, Project ID, target Account/Organization, previous role or owner, new role or owner, timestamp, and request correlation identifier for:

- invitations and revocations;
- joins, leaves, and removals;
- role changes;
- archive/restore;
- ownership transfer.

Audit records must not contain credentials, invitation secrets, or memory content.

### Naming

- Project slugs are unique within the owner namespace, not globally.
- Immutable identity uses `project_id`; owner/slug is presentation and lookup.
- Rename and transfer must not rewrite Project-scoped memory records.

### Durable Data

- Membership deletion does not delete Account identity or Project data.
- Project archive is reversible and read-only.
- [TBD: retention, export, and irreversible deletion policy. Until decided, version 1 must not expose permanent Project deletion.]

## 11. Non-Goals and Exclusions

- No Git staging, commit, branch, merge, push, pull, fetch, clone, or content-versioning implementation.
- No assumption that Project is local-only or cloud-only.
- No access grant based solely on filesystem possession or a local binding.
- No co-owner list; ownership is one Account or Organization namespace.
- No Organization team model in version 1.
- No public Project discovery or anonymous access.
- No Account-global or Organization-global memory scope.
- No billing, subscriptions, or Organization lifecycle implementation.
- No compatibility alias for rejected or experimental permission commands.
- No permanent Project deletion until retention and recovery policy is approved.

## 12. Acceptance Criteria

### Ownership

- [ ] `predicate: project.owner_type in {account, organization}`
- [ ] `predicate: exactly_one(project.owner_id)`
- [ ] `predicate: transfer(project).project_id == original.project_id`
- [ ] `predicate: transfer(project).memory_project_ids == original.memory_project_ids`
- [ ] `predicate: owner_account_cannot_leave_without_transfer_or_archive`

### Membership

- [ ] `predicate: invitation_recipient_must_match_authenticated_account`
- [ ] `predicate: invitation_acceptance_is_single_use`
- [ ] `predicate: only_owner_can_invite_remove_or_change_member_role`
- [ ] `predicate: removed_final_grant_denies_next_authorized_request`
- [ ] `predicate: removal_does_not_delete_project_memories`
- [ ] `predicate: inherited_only_member_cannot_leave_project_directly`

### Memory Isolation

- [ ] `predicate: every_persisted_memory_has_valid_project_id`
- [ ] `predicate: inaccessible_project_memory_read_is_denied`
- [ ] `predicate: inaccessible_project_memory_write_is_denied`
- [ ] `predicate: local_binding_without_server_grant_is_denied`
- [ ] `predicate: project_rename_or_transfer_does_not_rewrite_memory_scope`

### CLI

- [ ] Given a directory below a linked root, `sno project status` resolves the same `project_id`.
- [ ] Given `--project`, one command uses that accessible Project without changing the local binding.
- [ ] Given no explicit Project and no local binding, a Project-required command exits with a usage error.
- [ ] JSON output contains stable IDs and roles but no credential or invitation secret.
- [ ] Ownership and archive commands cannot proceed non-interactively without explicit confirmation.

### Concurrency and Audit

- [ ] `predicate: concurrent_transfer_produces_one_winner_and_one_conflict`
- [ ] `predicate: concurrent_invitation_acceptance_creates_at_most_one_membership`
- [ ] `predicate: every_membership_or_ownership_mutation_has_one_audit_record`
- [ ] `predicate: audit_record_contains_no_secret_or_memory_content`

## 13. Implementation Phases

### Phase 1: Identity and Authorization Contract

**Scope**: Immutable Project identity, owner namespace union, three-role permission model, direct grants, effective-permission function, memory `project_id` requirement, audit event schema.

**Validation**:

- property tests for exactly one owner;
- authorization table tests for every role/action pair;
- cross-Project memory isolation tests;
- concurrent mutation tests.

**Do not implement**: Organization teams, permanent deletion, public access.

### Phase 2: Personal Projects and Direct Membership

**Scope**: Account-owned creation, direct invitations, join/leave/remove, role changes, archive/restore, local binding resolution.

**Validation**:

- real API and persistent-store integration tests;
- invitation recipient and replay tests;
- immediate revocation tests;
- CLI/API parity tests.

### Phase 3: Organization Ownership

**Scope**: Organization-owned Projects, inherited Organization access, outside direct members, policy restrictions, personal-to-Organization transfer.

**Validation**:

- direct versus inherited grant precedence;
- loss of Organization membership;
- Organization policy denial;
- transfer preservation and conflict tests.

### Phase 4: Unified CLI Surface

**Scope**: Project status/init/link/unlink, member management, join/leave, transfer, deterministic JSON, explicit destructive confirmations.

**Validation**:

- actor-flow end-to-end tests;
- current-directory resolution tests across filesystem boundaries;
- secret-absence and output-schema tests;
- non-interactive fail-closed tests.

## 14. Agent Instructions

Always:

- use `project_id` for stored relationships and authorization;
- compute permission on the server from authoritative grants;
- preserve Project identity and memory scope during rename and transfer;
- add real persistence and concurrency tests for durable-data changes;
- keep CLI, API, and future user-interface permission semantics identical.

Ask before:

- adding or removing a Project role;
- allowing permanent Project deletion;
- adding public or anonymous Project access;
- adding Organization teams or custom roles;
- creating a new memory scope outside Project.

Never:

- infer access from a local directory or binding;
- trust client-submitted roles;
- create multiple owner namespaces;
- add Git content/version-control operations;
- silently preserve access after the final grant is revoked;
- log credentials, invitation secrets, or memory content in authorization evidence.

## 15. Open Decisions

1. Invitation expiry duration.
2. Exact Organization administrator-to-Project role mapping.
3. Project-managed file types covered alongside memories.
4. Archive retention, restoration window, export, and permanent deletion policy.
These decisions do not change the settled core: one Account-or-Organization owner namespace, exactly three Project roles, repository-like Project identity, direct and inherited membership, Project-scoped memory, and no Git content synchronization.

## 16. Reference Model

- GitHub personal repositories use one personal owner plus collaborators: [Permission levels for a personal account repository](https://docs.github.com/en/repositories/managing-your-repositorys-settings-and-features/repository-access-and-collaboration/permission-levels-for-a-personal-account-repository).
- Organization repositories grant roles to members, outside collaborators, and teams: [Repository roles for an organization](https://docs.github.com/en/organizations/managing-user-access-to-your-organizations-repositories/managing-repository-roles/repository-roles-for-an-organization).
- Repositories may transfer between personal and Organization owners while preserving the resource: [Transferring a repository](https://docs.github.com/en/repositories/creating-and-managing-repositories/transferring-a-repository).

These references inform the user mental model. SNO's authorization behavior is defined only by this PRD.
