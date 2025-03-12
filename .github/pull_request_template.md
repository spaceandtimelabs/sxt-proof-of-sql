Please be sure to look over the pull request guidelines here: https://github.com/spaceandtimelabs/sxt-proof-of-sql/blob/main/CONTRIBUTING.md#submit-pr.

# Please go through the following checklist
- [ ] The PR title and commit messages adhere to guidelines here: https://github.com/spaceandtimelabs/sxt-proof-of-sql/blob/main/CONTRIBUTING.md. In particular `!` is used if and only if at least one breaking change has been introduced.
- [ ] I have run the ci check script with `source scripts/run_ci_checks.sh`.
- [ ] I have run the clean commit check script with `source scripts/check_commits.sh`, and the commit history is certified to follow clean commit guidelines as described here: https://github.com/spaceandtimelabs/sxt-proof-of-sql/blob/main/COMMIT_GUIDELINES.md
- [ ] The latest changes from `main` have been incorporated to this PR by simple rebase if possible, if not, then conflicts are resolved appropriately.

# Rationale for this change

<!--
 Why are you proposing this change? If this is already explained clearly in the linked issue then this section is not needed.
 Explaining clearly why changes are proposed helps reviewers understand your changes and offer better suggestions for fixes.

 Example:
 Add `NestedLoopJoinExec`.
 Closes #345.

 Since we added `HashJoinExec` in #323 it has been possible to do provable inner joins. However performance is not satisfactory in some cases. Hence we need to fix the problem by implement `NestedLoopJoinExec` and speed up the code
 for `HashJoinExec`.
-->

# What changes are included in this PR?

<!--
There is no need to duplicate the description in the ticket here but it is sometimes worth providing a summary of the individual changes in this PR.

Example:
- Add `NestedLoopJoinExec`.
- Speed up `HashJoinExec`.
- Route joins to `NestedLoopJoinExec` if the outer input is sufficiently small.
-->

# Are these changes tested?
<!--
We typically require tests for all PRs in order to:
1. Prevent the code from being accidentally broken by subsequent changes
2. Serve as another way to document the expected behavior of the code

If tests are not included in your PR, please explain why (for example, are they covered by existing tests)?

Example:
Yes.
-->
