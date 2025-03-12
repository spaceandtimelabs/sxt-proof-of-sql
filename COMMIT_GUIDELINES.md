We follow conventional commits as defined [here](https://www.conventionalcommits.org/en/v1.0.0/#summary). We (and most major projects) will not merge your PR unless your commit history aligns to these guidelines.

This document explains how to clean up your commit history, squash commits into a single commit if needed, and ensure you adhere to our conventional commit style. We use the following standard prefixes:

- feat
- fix
- test
- docs
- refactor
- perf

## Example of a Messy Commit History

Consider a history like this:
```
9ec71a4 partial fix for issue #22 
4a8d2b6 correct typo in logging 
a19b75c Merge branch 'main' into feature-branch 
3b22d10 update docs again
```

We want to transform these into a single clean commit or a small number of commits with proper messages. At the end, we want only this:

```
9ec71a4 fix: partial fix for issue #22 
```

## Step-by-Step: Squashing and Cleaning Up Commits

1. **Update your local branch**  

```
git checkout feature-branch 
git fetch origin 
git rebase origin/main
```

2. **Start interactive rebase**  
Identify how many commits need adjusting (in this example, 4). Then run:  

```
git rebase -i HEAD~4
```

A text editor opens with a list of commits.

3. **Choose your actions**  
In the editor, specify one commit as `pick` (the first commit you want to keep) and mark the others as `squash` or `fixup`:

```
pick 9ec71a4 partial fix for issue #22 
squash 4a8d2b6 correct typo in logging 
squash a19b75c Merge branch 'main' into feature-branch 
squash 3b22d10 update docs again
```

- `pick`: Keep the commit as is.
- `squash`: Combine this commit with the previous commit and let you edit the commit message.
- `fixup`: Combine this commit with the previous commit but use the previous commit’s message.

4. **Reword the commit**  
After saving the rebase file, a new editor window will appear if you used `squash`. Enter a new commit message that follows our style:

```
feat: add documentation and fix logging
```

(Detailed description about what was changed and why.)


5. **Resolve Conflicts if Needed**  
If there are merge conflicts, edit the files to fix them, then do:

```
git add <file1> <file2> ... git rebase --continue
```

6. **Force push your changes**  
After the rebase completes:

```
git push -f origin feature-branch
```

**Note**: This overwrites the remote history for your branch, so be cautious when others are working on the same branch.

## Final Commit Message Examples

Here are some acceptable final commit messages:

- `feat: implement new user login`
- `fix: resolve null pointer exception`
- `test: add tests for new user login`
- `docs: update README with usage instructions`
- `refactor: simplify database query logic`
- `perf: improve caching mechanism for faster responses`

Always keep your commit message clear and concise. You should add an exclamation mark `!` after the type if it’s a breaking change, for example:

```
feat!: remove deprecated authentication method
```

By following these steps, you’ll ensure that your commit history is clean, meaningful, and easy to review.

This is a highly simplified guide to clean commits. The full docs for this can be found in our contributing guidelines [here](https://github.com/angular/angular/blob/main/CONTRIBUTING.md).