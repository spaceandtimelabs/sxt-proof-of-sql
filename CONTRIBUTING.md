<h1 align="center" style="border-bottom: none;">Contributing to our Proofs Repository</h1>
<p align="center">
  <a href="https://www.conventionalcommits.org/en/v1.0.0/">
    <img alt="semantic-release: conventional-commits" src="https://img.shields.io/badge/semantic--release-conventional--commits-blueviolet">
  </a>
</p>

The following guideline is heavily based on the [Angular Project Guideline](https://github.com/angular/angular/blob/main/CONTRIBUTING.md). As a contributor, here are the rules we would like you to follow:

 - [Submission Guidelines](#submit)
   - [Submitting a Pull Request (PR)](#submit-pr)
   - [Addressing review feedback](#address-review)
   - [Updating the commit message](#updating-commit-message)
   - [After your pull request is merged](#after-pr-merged)
 - [Coding Rules](#rules)
 - [Commit Message Guidelines](#commit-guidelines)
   - [Commit Message Format](#commit)
   - [Commit Message Header](#commit-message-header)
     - [Type](#type)
     - [Summary](#summary)
   - [Commit Message Body](#commit-message-body)
   - [Commit Message Footer](#commit-message-footer)
   - [Revert Commits](#revert)
   - [Commit Examples](#commit-examples)
   - [Automatic Semantic Release](#semantic-release)


## <a name="submit"></a> Submission Guidelines

This project is built using the [Cargo build system](https://doc.rust-lang.org/book/ch01-03-hello-cargo.html).

The general flow you can follow when implementing a new feature/bugfix/docs/test is:

1. Create a GitHub issue to keep track of the task you are implementing.
 
The most relevant fields in the issue are: `assignees`, `projects`, `milestone`, `development`, and `description`. Those fields are not mandatory, but they may help in the future to easily fetch information related to a specific feature, such as the time it took from implementation until completeness, and which PRs are associated with which feature (many PRs can be related to a single feature/issue).

2. From the created issue panel, use the `main` branch to generate a new branch that will be tied with the issue. In this case, when a Pull Request tied with the branch is merged, the issue will be automatically closed.

3. As a convention, you can append the related problem you are trying to solve to the branch name. For instance:

```
feat/compute-commitments
```

```
fix/compute-commitments
```

```
docs/compute-commitments
```

```
ci/set-up-environment
```

4. Whenever you are done implementing the modifications in your branch, make a Pull Request to merge your changes into the main branch. Try to always assign someone to review your Pull Request. Since we are using an automatic release process to version our code, you should follow a strict pattern in your commit messages (below for more descriptions). It is advised that you name your Pull Request according to our semantic release rules, given that the commit message is automatically the same as the Pull Request title. For instance, name the PR as "feat: add hadamard product" and do not name the PR as "Adding hadamard product". Always test your code locally before any pull request is submitted.

5. In the case of many commit messages to your branch, force the Pull Request to merge as a squashed merge.

6. After the merge is done, delete your branch from the repository and check that the related issue was indeed closed.

### <a name="submit-pr"></a> Submitting a Pull Request (PR)

Before you submit your Pull Request (PR) consider the following guidelines:

1. Make your changes in a new git branch:

   In case you haven't generated a new branch yet, use the following command to create a new branch from the main:
     ```shell
     git checkout -b my-feature-branch main
     ```

  Otherwise, only checkout your branch:

    ```shell
     git checkout my-feature-branch
     ```

2. Create your patch, **including appropriate test cases**.

3. Follow our [Coding Rules](#rules).

4. Run the entire test suite to ensure tests are passing.

    ```shell
    cargo test
    ```

5. Commit your changes using a descriptive commit message that follows our [commit message conventions](#commit). Adherence to these conventions is necessary because release notes are automatically generated from these messages.

     ```shell
     git add <modified files>
     git commit
     ```

    Note: Only add relevant files. Avoid adding binary files, as they frequently waste storage resources. Consider adding only text files (.rs, .cc, .json, .toml, etc).

6.  Push your branch to GitHub:

    ```shell
    git push origin my-feature-branch
    ```

7.  In GitHub, send a pull request to `proofs:main`.

Our proofs repository triggers automatically a workflow to test the code whenever a Pull Request is submitted or a commit is pushed to an existing PR. Before closing the PR, always verify that those tests are indeed passing.

Also, to ease this process of using git, you can try to use [vscode](https://code.visualstudio.com/). Vscode has some nice extensions to manage your git workflow.

### <a name="address-review"></a> Addressing review feedback

If we ask for changes via code reviews then:

1. Make the required updates to the code.

2. Re-run the entire test suite to ensure tests are still passing.

3. Create a fixup commit and push to your GitHub repository (this will update your Pull Request):

    ```shell
    # Create a fixup commit to fix up the last commit on the branch:
    git commit --all --fixup HEAD
    git push
    ```

    or

    ```shell
    # Create a fixup commit to fix up commit with SHA <COMMIT_SHA>:
    git commit --fixup <SHA>
    ```

    For more info on working with fixup commits see [here](https://github.com/angular/angular/blob/main/docs/FIXUP_COMMITS.md).

4. When merging the PR, try to choose the squashed merge version as it does not pollute the main branch with many commit messages.

### <a name="updating-commit-message"></a> Updating the commit message

A reviewer might often suggest changes to a commit message (for example, to add more context for a change or adhere to our [commit message guidelines](#commit)).
In order to update the commit message of the last commit on your branch:

1. Check out your branch:

    ```shell
    git checkout my-fix-branch
    ```

2. Amend the last commit and modify the commit message:

    ```shell
    git commit --amend
    ```

3. Push to your GitHub repository:

    ```shell
    git push --force-with-lease
    ```

NOTE: If you need to update the commit message of an earlier commit, you can use `git rebase` in interactive mode. See the [git docs](https://git-scm.com/docs/git-rebase#_interactive_mode) for more details.


### <a name="after-pr-merged"></a> After your pull request is merged

After your pull request is merged, you can safely delete your branch and pull the changes from the main (upstream) repository:

* Delete the remote branch on GitHub either through the GitHub web UI or your local shell as follows:

    ```shell
    git push origin --delete my-fix-branch
    ```

* Check out the main branch:

    ```shell
    git checkout main -f
    ```

* Delete the local branch:

    ```shell
    git branch -D my-fix-branch
    ```

* Update your local `main` with the latest upstream version:

    ```shell
    git pull --ff upstream main
    ```


## <a name="rules"></a> Coding Rules
To ensure consistency throughout the source code, keep these rules in mind as you are working:

* All features or bug fixes **must be tested** by one or more specs (unit-tests). 
* All public API methods **must be documented**. We follow the rust documentation style (see [here](https://doc.rust-lang.org/cargo/commands/cargo-doc.html)).

## <a name="commit-guidelines"></a> Commit Message Guidelines

### <a name="commit"></a> Commit Message Format

*This specification is inspired by and supersedes the
[Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/).*

We have very precise rules over how our Git commit messages must be formatted.
This format leads to **easier to read commit history** and also smooths our **automatic release process**.

Each commit message consists of a **header**, a **body**, and a **footer**.

```
<header>
<BLANK LINE>
<optional body>
<BLANK LINE>
<optional footer>
```

The `header` is mandatory and must conform to the [Commit Message Header](#commit-header) format.

The `body` is optional for all commits.
When the body is present it must conform to the [Commit Message Body](#commit-body) format.

The `footer` is optional. The [Commit Message Footer](#commit-footer) format describes what the footer is used for and the structure it must have.


#### <a name="commit-header"></a>Commit Message Header

```
<type>: <short summary>
  │           │
  │           └─⫸ Summary in present tense. Not capitalized. No period at the end.
  │
  │
  └─⫸ Commit Type: feat|fix|perf|refactor|test|build|ci|docs|style|chore
```

Both `<type>` and `<summary>` fields are mandatory. `Type` must always be followed by a `:`, a space, then the `summary`. Optionally, you can add a `!` before the `:` so that the release analyzer can be aware of a breaking change, thus allowing the bump of the major version.

#### <a name="type"></a> Type

Must be one of the following:

* **feat**: A new feature
* **fix**: A bug fix
* **perf**: A code change that improves performance
* **refactor**: A code change that neither fixes a bug nor adds a feature
* **test**: Adding missing tests or correcting existing tests
* **bench**: Adding missing benchmarks or correcting existing benchmarks
* **build**: Changes that affect the build system or external dependencies (example scopes: gulp, broccoli, npm)
* **ci**: Changes to our CI configuration files and scripts
* **docs**: Documentation only changes
* **style**: Feature and updates related to styling
* **chore**: Regular code maintenance.

Try to not fill your commit with many unrelated changes to your code, as it makes the process of review more difficult. For instance, if you add a feature and tests to validate your feature, try to commit your code as two messages, one for the feature implementation ("feat: add feature x") and another for the test addition ("test: add tests to validate feature x").

#### <a name="summary"></a>Summary

Use the summary field to provide a succinct description of the change (less than 80 characters):

* use the imperative, present tense: "change", not "changing", nor "changed", and nor "changes"
* don't capitalize the first letter
* no dot (.) at the end

### <a name="commit-body"></a>Commit Message Body

Just as in the summary, use the imperative, present tense: "fix", not "fixed", nor "fixes", neither "fixing".

Explain the motivation for the change in the commit message body. This commit message should explain _why_ you are making the change.
You can include a comparison of the previous behavior with the new behavior in order to illustrate the impact of the change.

### <a name="commit-footer"></a>Commit Message Footer

The footer can contain information about breaking changes and deprecations and is also the place to reference GitHub issues and other PRs that this commit closes or is related to. For example:

```
<feat | perf | fix>: <change summary>
<BLANK LINE>
<breaking change description + migration instructions>
<BLANK LINE>
BREAKING CHANGE: Fixes #<issue number>
```

Breaking Change section must always be at the message footer.

### <a name="revert"></a>Revert commits

If the commit reverts a previous commit, it should begin with `revert: `, followed by the header of the reverted commit.

The content of the commit message body should contain:

- information about the SHA of the commit being reverted in the following format: `This reverts commit <SHA>`,
- a clear description of the reason for reverting the commit message.

## <a name="commit-examples"></a>Commit Examples

### Commit message with ! to draw attention to breaking change

```
feat!: send an email to the customer when a product is shipped
```

### Commit message with both ! and BREAKING CHANGE footer

```
chore!: drop support for Node 6

BREAKING CHANGE: use JavaScript features not available in Node 6.
```

### Commit message with description and breaking change in the footer

```
feat: allow provided config object to extend other configs

BREAKING CHANGE: `extends` key in config file is now used for extending other config files
```

### Commit message with no body

```
docs: correct spelling of CHANGELOG
```

### Commit message for a fix using an (optional) issue number.

```
fix: minor typos in code

see the issue for details on the typos fixed

fixes issue #12
```


## <a name="semantic-release"></a>Automatic Semantic - Release process

We are using a node semantic-release tool to automatically trigger our release process. As shown below, this tool inspects the commitment message to decide if the release should be triggered and which type of release should be triggered:

| Type     | Message                                                                                                                                                                                       | Release Type                                                                                                  |
| -------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------- |
| ci       | ci:                                                                                                                                                                                           | No Release                                                                                                    |
| docs     | docs:                                                                                                                                                                                         | No Release                                                                                                    |
| refactor | refactor:                                                                                                                                                                                     | No Release                                                                                                    |
| test     | test: add new unit tests to gpu commitment module                                                                                                                                             | No Release                                                                                                    |
| build    | build:                                                                                                                                                                                        | Fix Release (Patch)                                                                                                    |
| perf      | perf: speedup gpu commitment by 3x                                                                                                                                    | Fix Release (Patch)                                                                                           |
| fix      | fix: stop graphite breaking when too much pressure applied                                                                                                                                    | Fix Release (Patch)                                                                                           |
| feat     | feat: graphiteWidth' option                                                                                                                                                                   | Feature Release (Minor)                                                                                       |
| feat     | feat: add graphiteWidth option<br><br><body> The default graphite width of 10mm is always used for performance reasons.<br><br>BREAKING CHANGE: The graphiteWidth option has been added. | Breaking Release (Major)<br><br>(Note that the BREAKING CHANGE:<br>token must be in the footer of the commit) |
| perf     | perf: remove graphiteWidth option<br><br><body> The default graphite width of 10mm is always used for performance reasons.<br><br>BREAKING CHANGE: The graphiteWidth option has been removed. | Breaking Release (Major)<br><br>(Note that the BREAKING CHANGE:<br>token must be in the footer of the commit) |

Check the [Semantic-Release](https://github.com/semantic-release/semantic-release) link for more info.

