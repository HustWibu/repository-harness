# Repository Understanding

Repository code, types, module interfaces, executable commands, tests, examples,
and runtime evidence hold current implementation knowledge. Product intent,
rationale, and accepted constraints remain with their repository owners.
Use `docs/README.md` when present, otherwise inspect native sources; link to existing knowledge
rather than reproduce it in a new summary.

## Authority And Scope

Respect the user's requested scope. Discovery does not authorize editing,
starting services, installing tools, or changing external settings.
Before changing externally observable policy, identify accepted repository
authority. If materially different choices remain, stop and request the smallest
decision. Code and tests show current behavior; configurable defaults are not
authority for missing product policy.
Configuration checked into a repository may describe only a development
environment. Before concluding how an environment behaves, find its
configuration source under Authority By Truth Type in `docs/README.md`.
When existing code and a written rule disagree, consistency (names, layering,
base libraries) follows the existing code; correctness and safety follow the
rule, and fixing old code is a separate change.

## Missing Capabilities

Identify the capabilities needed for the requested outcome: locating an owner
or interface, running a command, preparing known state, observing an effect,
verifying behavior, or recovering safely. Check the relevant implementation,
commands, fixtures, logs, and environment rather than assuming they exist.
Report an unavailable capability with its evidence and owning repository or
external dependency. Keep unobserved state Unknown. When the answer belongs to
another repository, name that owner from Related Repositories in
`docs/README.md` rather than leaving an ownerless Unknown.

Application commands, credentials, readiness, state ownership, and cleanup
belong to the consumer repository. A template supplies structure, not proof
that an application can be operated. Do not invent missing commands or policy.

## Trust Through Evidence

Use repository-native checks and observations appropriate to the claimed
behavior. Protect accepted safety and correctness boundaries; preserve existing
state and verified recovery mechanisms. Distinguish observed results from
assumptions, and structural checks from semantic or runtime proof.

A plan, checklist, or completion message is not behavior-level proof. A checked-in
CI job does not prove it ran or blocks merging. Report what was exercised, what
failed or could not run, and what remains unverified.
For facts or contracts that span repositories, follow
`docs/patterns/multi-repository-evidence.md`.

## Definition Of Done

Work is done only when evidence matching the changed behavior was observed.
Report three lists: verified, with the command or observation; not verified,
with the reason; and known broken. Missing evidence means not verified, never
done.

Give each verified item its evidence scope: in-repository, across a contract,
or on a deployed environment. When a change touches a contract and only
in-repository evidence exists, list its consumers under not verified with the
reason. The consumer repository decides the minimum scope for each kind of
change.

## Optional Resources

Select relevant Harness skills from the requested outcome; names are optional.
Load only skills that materially help the current task. Skill selection grants
no additional authority to edit, operate, or publish. Consumer instructions
own the task workflow; templates and plans remain optional. When a repository
has no Harness documents, inspect its native instructions, source, and checks
rather than creating missing context files automatically.
