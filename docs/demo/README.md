# Repository Understanding Examples

These examples show how repository knowledge and evidence support different
requests. They illustrate choices a consumer repository or user can make;
Harness does not prescribe task categories or a planning lifecycle.

Assume a small team task tracker with this product rule in
`docs/product/tasks.md`:

```text
A task has a title, status, assignee, and optional due date.
Supported statuses are todo, in_progress, done, and canceled.
Only an unfinished task whose due date has passed is overdue.
```

## 1. Read-Only Question

Request:

```text
When does a task become overdue?
```

Step by step:

1. Read `CLAUDE.md`, which points to the repository map.
2. Open the relevant product contract, `docs/product/tasks.md`.
3. Answer from that rule and cite the file.
4. Do not bootstrap a database, create an intake, write a trace, or modify the
   repository.

Cause and effect: the question needs evidence, not durable workflow state. A
read-only path makes the answer faster and prevents an explanation from
silently mutating the project.

## 2. Bounded Change

Request:

```text
Fix the task list so canceled tasks are not marked overdue.
```

Step by step:

1. Read the overdue rule and locate the task-list calculation.
2. Inspect the nearest tests and repository validation command.
3. Keep a short working plan in the current session.
4. Change the calculation to require an unfinished, non-canceled task.
5. Add or update a regression test for a canceled task with a past due date.
6. Run the focused test, then the repository's relevant validation gate.
7. Report the changed behavior and proof.

Cause and effect: the scope is local and recoverable from the diff. Creating a
durable plan or database row would add synchronization work without preserving
information that Git and the test do not already contain.

## 3. Change Across Several Boundaries

Request:

```text
Replace local due-date handling with team time zones across the API, worker,
UI, and stored data.
```

Step by step:

1. Inspect the product, architecture, migration, and validation surfaces.
2. Establish authority for the time-zone model and migration behavior before
   changing externally observable policy.
3. Identify missing capabilities for migration, application observation,
   validation, and recovery; report anything unavailable or unproven.
4. Follow the consumer repository's chosen workflow. If the user or repository
   chooses a durable plan, `docs/templates/exec-plan.md` is an optional starting
   point for goals, boundaries, risks, recovery, and proof commands.
5. Implement in reviewable groups and run the relevant checks.
6. Preserve accepted intent and rationale that future work cannot recover from
   code, using the repository's chosen record location.
7. Run end-to-end proof across the visible application boundary and report its
   limits.

Cause and effect: this change spans boundaries. Source references, executable
checks, and observable recovery establish what is understood and verified.
A versioned plan can preserve sequencing when the consumer chooses to use one.

## 4. Consequential Ambiguity

Request:

```text
Simplify task permissions.
```

The repository reveals at least two plausible interpretations:

- allow every teammate to edit every task; or
- keep ownership restrictions but simplify the permission code.

Step by step:

1. Inspect the current permission contract and callers.
2. Identify that one interpretation changes who may modify user data.
3. Pause before editing code.
4. Present the two choices with concrete effects: access expansion versus an
   implementation-only refactor.
5. Continue only when the requested product behavior is authoritative.

Cause and effect: uncertainty is not solved by adding more process records. It
is solved by keeping a consequential product decision with the human who owns
it.

## What Is Deliberately Absent

None of these examples requires a story row, proof matrix, trace score,
audit record, proposal, or parallel task database. Those are not part of the
current product and do not sit between a normal request and repository work.
