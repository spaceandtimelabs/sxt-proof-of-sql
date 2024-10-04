<h1 align="center" style="border-bottom: none;">Contributing to our Proof of SQL Repository</h1>
<p align="center">
  <a href="https://www.conventionalcommits.org/en/v1.0.0/">
    <img alt="semantic-release: conventional-commits" src="https://img.shields.io/badge/semantic--release-conventional--commits-blueviolet">
  </a>
</p>

The following guideline is heavily based on the [Angular Project Guideline](https://github.com/angular/angular/blob/main/CONTRIBUTING.md). As a contributor, here are the rules we would like you to follow:

 - [Getting Started](#getting-started)
   - [Install Rust](#install-rust)
   - [Fork the Repository](#fork-the-repository)
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

## <a name="getting-started"></a> Getting Started
### <a name="install-rust"></a> Install Rust
To contribute to this project, you'll need to have Rust installed on your machine. Follow the steps below to install Rust and set up your development environment:

1. Install Rust.
   - Rust's official installer is called `rustup`, which makes it easy to install and manage Rust versions.
   - To install Rust, open your terminal and run the following command:
     ```bash
     curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
     ```
   - Follow the on-screen instructions to complete the installation.

2. Verify your installation.
   - Once Rust is installed, verify the installation by checking the version:
     ```bash
     rustc --version
     ```
   - This should display the version of Rust that was installed.

3. Set up your environment.
   - After installation, `rustup` will configure your environment automatically, adding Rust to your system's PATH. If needed, you can reload your shell with:
     ```bash
     source $HOME/.cargo/env
     ```

4. Update Rust.
   - To ensure you are using the latest version of Rust, you can update Rust at any time by running:
     ```bash
     rustup update
     ```

5. Additional tools.
   - You may also want to install some common tools used in Rust development, like `cargo` (Rust’s package manager and build system) and `rustfmt` (code formatting):
     ```bash
     rustup component add rustfmt
     rustup component add clippy
     ```

6. Start building.
   - With Rust installed, you're now ready to start developing! You can create a new project by running:
     ```bash
     cargo new my_project
     cd my_project
     ```

If you run into any issues, please refer to the [official Rust documentation](https://www.rust-lang.org/learn/get-started) for troubleshooting and more detailed installation instructions.

### <a name="fork-the-repository"></a> Fork the Repository
In order to contribute it is necessary to fork the repository. Follow the steps below to do so:

1. Navigate to the repository.
   - Go to the main page of the repository on GitHub at https://github.com/spaceandtimelabs/sxt-proof-of-sql.

2. Fork the repository.
   - Click the "Fork" button in the upper-right corner of the repository page. This will create a copy of the repository under your GitHub account.

3. Clone your forked repository.
   - Once the fork is complete, you need to clone it to your local machine. Run the following command in your terminal:
     ```bash
     git clone https://github.com/YOUR_USERNAME/sxt-proof-of-sql.git
     ```
   - Replace `YOUR_USERNAME` with your GitHub username.

4. Navigate to the repository's directory.
   - Change into the newly cloned repository directory:
     ```bash
     cd sxt-proof-of-sql
     ```

5. Add the original repository as a remote (optional but recommended).
   - This allows you to keep your fork up to date with changes from the original repository:
     ```bash
     git remote add upstream https://github.com/spaceandtimelabs/sxt-proof-of-sql.git
     git remote add origin https://github.com/YOUR_USERNAME/sxt-proof-of-sql.git
     ```
   - Replace `YOUR_USERNAME` with your GitHub username.

6. Start making your changes.
   - Now you are ready to create a new branch and make your contributions!

## <a name="submit"></a> Submission Guidelines

This project is built using the [Cargo build system](https://doc.rust-lang.org/book/ch01-03-hello-cargo.html).

The general flow you can follow when implementing a new feature/bugfix/docs/test is:

1. Create a GitHub issue to keep track of the task you are implementing.
 
The most relevant fields in the issue are: `assignees`, `projects`, `milestone`, `development`, and `description`. Those fields are not mandatory, but they may help in the future to easily fetch information related to a specific feature, such as the time it took from implementation until completeness, and which PRs are associated with which feature (many PRs can be related to a single feature/issue).

2. From the created issue panel, use the `main` branch to generate a new branch that will be tied with the issue. In this case, when a Pull Request tied with the branch is merged, the issue will be automatically closed.

3. Whenever you are done implementing the modifications in your branch, make a Pull Request to merge your changes into the main branch. Try to always assign someone to review your Pull Request. Since we are using an automatic release process to version our code, you should follow a strict pattern in your commit messages (below for more descriptions). It is advised that you name your Pull Request according to our semantic release rules, given that the commit message is automatically the same as the Pull Request title. For instance, name the PR as "feat: add hadamard product" and do not name the PR as "Adding hadamard product". Always test your code locally before any pull request is submitted.

4. In the case of many commit messages to your branch, force the Pull Request to merge as a squashed merge.

5. <a name="delete-merged-branch-and-check-related-issues"></a>After the merge is done, delete your branch from the repository and check that related issues were indeed closed.

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

4. <a name="test-suite"></a>Run the entire test suite to ensure tests are passing.

    ```shell
    cargo test --all-features
    ```

5. <a name="code-quality-checks"></a>Run the following code quality checks locally so that the code is not only correct but also clean.

    ```shell
    cargo fmt --all -- --config imports_granularity=Crate,group_imports=One --check
    cargo clippy --all-targets --all-features -- -D warnings
    ```

6. Commit your changes using a descriptive commit message that follows our [commit message conventions](#commit). Adherence to these conventions is necessary because release notes are automatically generated from these messages.

     ```shell
     git add <modified files>
     git commit
     ```

    Note: Only add relevant files. Avoid adding binary files, as they frequently waste storage resources. Consider adding only text files (.rs, .cc, .json, .toml, etc). Files that should NOT be committed should instead be added
    to `.gitignore`.

7.  Push your branch to GitHub:

    ```shell
    git push origin my-feature-branch
    ```

8.  In GitHub, send a pull request to `sxt-proof-of-sql:main`.

Our proof of SQL repository triggers automatically a workflow to test the code whenever a Pull Request is submitted or a commit is pushed to an existing PR. Before closing the PR, always verify that those tests are indeed passing.

NOTE: <a name="ci-review-note"></a>**We will not review a PR if CI (except for `Check Approver` since this requires a review) doesn't pass. We are happy to help you if you can't figure out how to get the CI to pass but it is your responsibility to make sure they pass.**

Also, to ease this process of using git, you can try to use [vscode](https://code.visualstudio.com/). Vscode has some nice extensions to manage your git workflow.

### <a name="address-review"></a> Addressing review feedback

If we ask for changes via code reviews then:

1. Make the required updates to the code.

2. [Re-run the entire test suite](#test-suite) to ensure tests are still passing.

3. [Re-run the code quality checks](#code-quality-checks) to ensure that the code is still clean.

4. Create a fixup commit and push to your GitHub repository (this will update your Pull Request):

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

5. In order to ensure that we do not pollute the main branch with poorly written commit messages, before the PR can be merged, we require that the commits in your branch be clean. In particular, this means that you should rebase instead of merge in order to catch up to main. Additionally, any commits of the variety `address review comments` should be turned into a fixup commit instead.

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

### <a name="semantic-version"></a> Semantic Versioning

To version our code, we follow an **automatic semantic versioning** given by the [Semantic Versioning](https://semver.org/) scheme, which establishes that the version is given by **"MAJOR.MINOR.PATCH"** number, which is updated as:

1. Increase the **MAJOR** version when you make incompatible API changes.
2. Increase the **MINOR** version when you add functionality in a backwards compatible manner.
3. Increase the **PATCH** version when you make backwards compatible bug fixes.

For instance: "1.1.3" is a program that is in the first major and minor version and the third patch version. When an incompatible change is done to the public API, then this version is updated to "2.0.0". If a backward compatible feature is added later, the version is updated to "2.1.0".

### <a name="commit"></a> Commit Message Format

*This specification is inspired by and supersedes the
[Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/).*

We have very precise rules over how our Git commit messages must be formatted.
This format leads to **easier to read commit history** and also smooths our **automatic semantic versioning**.

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
  |           |
  |           |
  |           |
  │           └─⫸ Summary in present tense. Not capitalized. No period at the end.
  │
  │
  └─⫸ Commit Type: feat|feat!|fix|fix!|perf|perf!|refactor|refactor!|test|bench|build|ci|docs|style|chore
```

Both `<type>` and `<summary>` fields are mandatory. `Type` must always be followed by a `:`, a space, then the `summary`. Optionally, you can add a `!` before the `:` so that the release analyzer can be aware of a breaking change, thus allowing the bump of the major version. For Rust please refer to [SemVer Compatibility in the Cargo book](https://doc.rust-lang.org/cargo/reference/semver.html) for what constitutes a breaking change.

#### <a name="type"></a> Type

Must be one of the following:

* **feat**: a commit of the type feat introduces a new feature to the codebase (this correlates with MINOR in Semantic Versioning).
* **feat!**: a commit of the type feat introduces a new feature to the codebase and introduces breaking changes (this correlates with MAJOR in Semantic Versioning).
* **fix**: a commit of the type fix patches a bug in your codebase (this correlates with PATCH in Semantic Versioning).
* **fix!**: a commit of the type fix a bug in your codebase and introduces breaking changes (this correlates with MAJOR in Semantic Versioning).
* **perf**: A code change that improves performance (this correlates with a PATCH in Semantic Versioning).
* **perf!**: A code change that improves performance and introduces breaking changes (this correlates with MAJOR in Semantic Versioning).
* **refactor**: A code change that neither fixes a bug nor adds a feature (this correlates with a PATCH in Semantic Versioning).
* **refactor!**: A code change that neither fixes a bug nor adds a feature and introduces breaking changes (this correlates with MAJOR in Semantic Versioning).
* **test**: Adding missing tests or correcting existing tests
* **bench**: Adding missing benchmarks or correcting existing benchmarks (this does not correlate with any semantic versioning update).
* **build**: Changes that affect the build system or external dependencies (this correlates with a PATCH in Semantic Versioning).
* **ci**: Changes to our CI configuration files and scripts.
* **docs**: Documentation only changes (this correlates with a PATCH in Semantic Versioning).
* **style**: Feature and updates related to styling (this does not correlate with any semantic versioning update).
* **chore**: Regular code maintenance (this does not correlate with any semantic versioning update).

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

## <a name="semantic-release"></a>Automatic Semantic - Release Process

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

Check the [Semantic-Release](https://github.com/semantic-release/semantic-release) link for more info. Ps: to update the above rules, check the [package.json](package.json) file, in the `release -> plugins -> @semantic-release/commit-analyzer -> releaseRules` section.

