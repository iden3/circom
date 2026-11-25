# `-cbc` examples

Circuits used while developing the certifiability checker
(`constraint_generation/src/certifier.rs`), kept here so each check (H1, H2,
H3, nested buses, arrays) can be reproduced without rebuilding them from
scratch. All of them run from the `circom_cbc` root using the binary
built at `target/debug/circom`.

## H3: real circomlib2, no flag needed

Several examples (`h3_trusted/`, `arrays/`, `nested_buses/pass_full_chain.circom`)
use `IsZero()` from the real circomlib2 (clone it first):

```
git clone --branch circomlib_buses https://github.com/iden3/circomlib.git /tmp/circomlib
```

then pass `-l /tmp/circomlib/circuits` when compiling (the examples were
checked against commit `aef239c`; `git -C /tmp/circomlib checkout aef239c`
to reproduce them exactly). No trust flag is needed: `-cbc` automatically
recognizes that the **definition** of `IsZero()` -- the `template IsZero()
{ ... }` text, with comments removed and whitespace runs collapsed -- has
the same hash as one of the 111 `circomlib_buses` templates (commit
`aef239c`) listed in `constraint_generation/src/circomlib2_template_hashes.txt`
(embedded as `KNOWN_CIRCOMLIB2_TEMPLATE_HASHES` in `certifier.rs`), and
treats it as a black box -- no matter which file or path it came from.

Trust is per template, not per file, and recursive: an instance is trusted
only if its own definition matches **and** every subcomponent it
instantiates is trusted too, i.e. its whole subtree of descendants is
pristine circomlib2:

- Re-indenting or re-commenting `IsZero()` keeps it trusted; changing any of
  its tokens makes it stop matching, so it falls through to ordinary
  checking (it fails H1 on its own `<--`, no separate rejection logic
  needed).
- Editing another template of `comparators.circom` does not affect
  `IsZero()`.
- If a descendant stops matching, every ancestor loses trust as well, even
  with its own text untouched: modify `Num2Bits` (in `bitify.circom`) and a
  `LessThan` that instantiates it is no longer trusted -- both are checked
  under H1/H2. The reverse does not hold: modifying `LessThan` leaves its
  pristine `Num2Bits` trusted. See `h3_trusted/per_template_lessthan.circom`.

### Refreshing the hashes: `-cbc --cbc-refresh`

The embedded hashes are frozen at whatever commit was vendored when
`circom` was built; `circomlib_buses` will keep moving. Passing
`--cbc-refresh` alongside `-cbc` makes the compiler re-clone
`circomlib_buses` with `git` (it fails with a clear message if `git` isn't in
`PATH` -- failing loudly there was an explicit requirement, not a silent
skip), parse every file with the compiler's own parser, hash every template
definition, and overwrite `.circomlib2_template_hashes` (`sha256sum`-style
format, one `<hash>  <path>::<template>` line per template) in the
directory `circom` is invoked from. Once that file exists, it is used
**exclusively** -- the embedded table is no longer consulted at all, on this
or any later run, for as long as the cache file is present. This is
deliberate: the whole point of refreshing is to replace a possibly
stale/compromised embedded snapshot with the current canonical truth, so an
old embedded hash for a template that has since been patched upstream (e.g.
a security fix) must stop being trusted rather than linger as a silent
fallback. Delete `.circomlib2_template_hashes` to go back to relying on the
embedded table. Refreshing needs network access and `git` installed -- which
is why it's a separate flag, never implied by plain `-cbc`. The cache file
has the same format as `circomlib2_template_hashes.txt`, so regenerating
the embedded table is just copying it there and rebuilding.

```
circom foo.circom -cbc --cbc-refresh -l /tmp/circomlib/circuits
```

## h1/ -- No Free Assignments

```
target/debug/circom certifier_examples/h1/pass_safe_mul.circom -cbc -o /tmp
target/debug/circom certifier_examples/h1/fail_free_div.circom -cbc -o /tmp
```

- `pass_safe_mul.circom`: certifies clean (`<==` everywhere).
- `fail_free_div.circom`: fails with `CE01`, pointing at the exact `<--` line,
  and with `CA01` (`Local signal out does not appear in any constraint`):
  `<--` adds no constraint, so the output is also structurally free.

## h2/ -- No Bare Tags

```
target/debug/circom certifier_examples/h2/fail_bare_tag.circom -cbc -o /tmp
target/debug/circom certifier_examples/h2/note_circom_already_checks_call_sites.circom -cbc -o /tmp
```

- `fail_bare_tag.circom`: fails with `CE02` -- a tag declared over an
  arithmetic expression, something circom itself lets through (nothing
  downstream ever consumes the tag) but that `-cbc` checks
  exhaustively regardless.
- `note_circom_already_checks_call_sites.circom`: **not** a `-cbc`
  example by itself -- it demonstrates why `check_subcomponent_input` was
  removed from the checker: the compiler itself already rejects (`T3001`,
  `AssignmentMissingTags`) feeding a subcomponent's tagged input a value that
  doesn't carry that tag, before `-cbc` even runs.

## h3_trusted/ -- Trusted Composition, with real circomlib2

```
target/debug/circom certifier_examples/h3_trusted/mixed_certified_trusted_broken.circom \
  -cbc -l /tmp/circomlib/circuits -o /tmp
```

`mixed_certified_trusted_broken.circom` shows all three possible outcomes at
once: `Adder()` (own code, certifies normally), `IsZero()` (real circomlib2,
uses `<--` internally, automatically treated as a black box via its hash),
and `Broken()` (own code with its own genuine H1 violation, always fails,
trusted or not: `CE01` for its `<--` plus `CA01` because its output `out`
ends up in no constraint).

```
target/debug/circom certifier_examples/h3_trusted/per_template_lessthan.circom \
  -cbc -l /tmp/circomlib/circuits -o /tmp
```

`per_template_lessthan.circom` uses `LessThan` (comparators.circom), which
instantiates `Num2Bits` (bitify.circom). Against pristine circomlib2 it
passes with `LessThan`, `Num2Bits` and `MaxbitCheck*` all trusted. Copy
`circuits/` and, in the copy, remove the line
`out[i] * (out[i] -1 ) === 0;` from `Num2Bits` only: compiling with `-l`
pointing at the copy, nothing is trusted any more. `Num2Bits(8)` and
`Num2Bits(9)` fail with `CE01` (their `<--`) and `CE02` (bare `binary` tag),
and `LessThan` and `MaxbitCheck`, whose own definitions are untouched but
which instantiate `Num2Bits`, lose trust too and fail `CE02` once checked.
If instead you only modify `LessThan` (e.g. `out <== 1 - n2b.out[n] * 1;`),
only `LessThan` loses trust; its `Num2Bits` children stay trusted.

## nested_buses/ -- nested buses (tagged fields inside a bus type)

```
target/debug/circom certifier_examples/nested_buses/fail_bare_nested_field.circom -cbc -o /tmp

target/debug/circom certifier_examples/nested_buses/pass_full_chain.circom \
  -cbc -l /tmp/circomlib/circuits -o /tmp
```

- `fail_bare_nested_field.circom`: `out.inner.x` (a field nested inside
  `Outer.inner: Inner`, tagged in the `Inner` type itself) is written from an
  arithmetic expression -- fails with `CE02`, citing `out.inner.x` exactly,
  not just `out`.
- `pass_full_chain.circom`: exercises, all honestly, the three ways a nested
  tag can be established: through a subcomponent's own nested bus output
  (`HonestNested`), writing straight into the nested field
  (`DirectFieldWrite`), and copying one whole bus into another locally
  (`LocalBusToBusCopy`, which propagates whatever was already proven for
  `src` into `dst`). It needs `IsZero()` (real circomlib2) to manufacture the
  first honest tag, same as the H3 example.

## arrays/ -- arrays of bus instances with tagged fields

Circom only confirms an array's tag "if it inherits the tag in all
positions" (verbatim comment in `assignment_utils.rs`,
`perform_tag_propagation`): a single dishonest position invalidates the
whole array, no matter how many other positions were honest.
`-cbc` follows the same rule -- every index of `arr[i].x` collapses
into a single tracked path `arr.x`, and it only takes one dishonest touch on
that path to permanently disqualify it (reported exactly once, at the first
bad touch found, regardless of order or how many good touches surround it).

```
target/debug/circom certifier_examples/arrays/fail_one_bad_element.circom \
  -cbc -l /tmp/circomlib/circuits -o /tmp

target/debug/circom certifier_examples/arrays/pass_all_elements_honest.circom \
  -cbc -l /tmp/circomlib/circuits -o /tmp
```

- `fail_one_bad_element.circom`: `arr[0].x` honest, `arr[1].x` dishonest ->
  exactly 1 error on `arr.x` (also tested in the reverse order during
  development: the error always appears exactly once, and a later honest
  touch never "rescues" an already-poisoned path).
- `pass_all_elements_honest.circom`: both elements honest -> certifies clean.

### `known_limitation_untouched_element.circom` -- passes on purpose

```
target/debug/circom certifier_examples/arrays/known_limitation_untouched_element.circom \
  -cbc -l /tmp/circomlib/circuits -o /tmp
```

`arr[1].x` is never touched anywhere in the file -- no `<==`, no `<--`,
nothing. H2 alone can't see it (it collapses with `arr[0].x` into the same
path `arr.x`, and `arr[0]`'s honest touch alone is enough to mark that path
verified). `CertificationResult::absorb_structural_warnings` re-runs the
same "unconstrained signal" analysis `--inspect` uses (regardless of whether
`--inspect` was passed) once the DAG is built, and that analysis does see
`arr[1].x` -- but only unconstrained **outputs** are promoted into
certification failures. Here `arr` is an intermediate signal that nothing
reads, so it cannot make any output non-deterministic and is deliberately
allowed. Expected result: PASS (`ArrOfBuses()` certified, `IsZero()`
trusted); `--inspect` still prints `[CA01] ... Local signal arr[1].x does not
appear in any constraint` as a warning.

### `known_limitation_untouched_output_across_component.circom` -- same field, but as an output

```
target/debug/circom certifier_examples/arrays/known_limitation_untouched_output_across_component.circom \
  -cbc -l certifier_examples/arrays -o /tmp
```

Here `arr` is an **output** of `ArrOfBuses` (not intermediate), and it's
`Consumer` (whoever instantiates it) that reads `c.arr[1].x` -- the
never-touched element -- to build its own output `out`. Reading an
uninitialized signal **from within the same template** that declares it is
caught by circom itself (a "trying to access to a signal that is not
initialized" error, before `-cbc` even runs); but reading it from
**outside**, through the component, goes through a different path (the
subcomponent's already-"exported" output storage) that doesn't go through
that check, and H2 doesn't catch it either, for the same collapsed-path
reason as the previous example. Caught by the structural analysis instead:
the DAG sees every concrete wire regardless of which side of a component
boundary declared it, and since `arr` is a declared output of `ArrOfBuses`,
its `CA01` is promoted. Expected result: `TagA()` (hand-written, uses `<--`,
unrelated to this file's point) fails on its own, and `ArrOfBuses()` fails
with `[CA01] ... Local signal arr[1].x does not appear in any constraint`.
`Consumer()` is not reported: its `CA02` (a father not using a
subcomponent's output) is never a certification failure by itself.

### `known_limitation_untouched_output_across_component_array.circom` -- scalar variant

```
target/debug/circom certifier_examples/arrays/known_limitation_untouched_output_across_component_array.circom \
  -cbc -o /tmp
```

Despite its name, here `arr` is a plain (non-array, non-bus) output of
`ArrOfBuses` that is never touched, read by `Consumer` through the
component. Expected result: `TagA()` fails with `CE01`/`CE02` (unrelated),
`ArrOfBuses()` with `[CA01] ... Local signal arr does not appear in any
constraint`, and `Consumer()` with `CE02` (its `{tagA}` output is copied
from `c.arr`, which carries no tag).
