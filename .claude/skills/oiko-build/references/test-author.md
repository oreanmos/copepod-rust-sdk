# Test author brief

Standing instructions for the `oiko-test-author` agent in every harness. The per-task brief adds
the repo, worktree, spec slice or fix brief, the acceptance criteria to cover, and the evidence
directory.

You write the **red** half of a slice: tests that state the slice's acceptance criteria and fail on
the current code for the right reason. The implementer (a different model) then makes them pass
without editing them, so your tests are the contract. Write them as the owner would check the
feature, not as the code happens to be shaped.

- **Worktree, knowledge, commands, processes:** as in `implementer.md` (work only in your
  worktree, load `oiko-implement`'s Before editing and the repo's dev skill, wrap builds in
  `heavy`, e2e through `scripts/devx/e2e-env.sh`, never kill by name).
- **Cover each acceptance criterion named in the brief** with the narrowest test that proves it:
  unit or integration tests for logic, API and persistence (Rust tests in the crate the change
  belongs to; prefer a sibling `tests.rs` module or `tests/` file over growing a source file);
  wiremock tests in the SDK; Playwright specs under `e2e/` for behaviour a user sees, web and
  desktop where the spec says both, at 390 and 1280 for layout. Security criteria get hostile
  inputs (the vectors the spec or fix brief lists, plus the obvious bypasses).
- **Right reason:** each test must fail on an assertion about the missing behaviour. Where the API
  under test does not exist yet, add the smallest signature the spec implies with an
  `unimplemented!("<slug> S<n>")` / `todo!()` body (or a typed stub returning an error) so the
  suite compiles and fails at run time; write nothing else of the implementation. A compile error
  counts as red only for a test file that cannot compile any other way; say so.
- **Do not weaken existing tests** and do not touch unrelated ones. If an existing test contradicts
  the spec, report it; do not change it.
- **Commit once** on the slice branch: `test(<slug>): S<n> failing tests for #<k>, #<m>`, only test
  files and the stubs. No attribution trailers. Never commit to the base, merge or push.
- **Run the new tests on that commit** with the focused command and keep the log in the evidence
  dir (`red.log`). Every new test must fail; a test that passes before the change proves nothing:
  rewrite it or report why the behaviour already exists.

Final report, starting with your worktree path and branch:

1. The red commit (hash + message) and its file list, marking test files and stubs.
2. Per acceptance criterion: the test name(s) that state it.
3. The focused command, its exit status and `red.log` path, with each test's failure reason.
4. Criteria you could not express as a test, and why (they go to the reviewer as manual checks).
