# Interface Contract: Name

Date checked: YYYY-MM-DD

<!--
Optional. Use it when another repository sends to or receives from this one,
for example through a message topic and schema, an RPC definition, or an HTTP
endpoint. The definition file or configuration key stays the source of record;
this page only points to it. Keep it in the contract directory named under
Authority By Truth Type in docs/README.md.
-->

## Contract

| Field | Value |
| --- | --- |
| Channel | Topic, RPC, endpoint, or other named channel |
| Direction | Produce or consume, from this repository's side |
| Defined in | Path to the definition file, or the configuration key |
| Owning repository | Repository that may change the definition |

## Known Counterparts

| Repository | Direction | Declared in | Checked on |
| --- | --- | --- | --- |

Generate this list from the real sources (configuration, code) with the
consumer repository's own tooling where possible. If it is written by hand,
record the date each row was checked against those sources; an unchecked row
is a guess.

## Compatibility Check

Name the repository-native check that proves a change stays compatible, where
it runs, and which counterpart checkouts it reads. Write "none" if no check
exists yet; that is a gap, not a pass.

## Allowed Changes

State which changes the owner accepts without coordinating counterparts, for
example adding an optional field, and which need every counterpart updated
first. Leave this empty until the owner decides; do not infer it from code.

## Unknowns

List counterparts, fields, or rules not yet established.
