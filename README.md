<div id="top"></div>

<!-- PROJECT LOGO -->
<br />
<div align="center">
  <h1 align="center">Proofs</h1>

  <a href="https://dl.circleci.com/status-badge/redirect/gh/spaceandtimelabs/proofs/tree/main">
    <img alt="Build State" src="https://dl.circleci.com/status-badge/img/gh/spaceandtimelabs/proofs/tree/main.svg?style=svg&circle-token=b65006a5aecc40183a7eaad478fbbcf7b0a50337">
  </a>
  <a href="https://spaceandtimeworkspace.slack.com">
    <img alt="Slack URL" src="https://img.shields.io/badge/slack-@spaceandtimeworkspace-yellow.svg?logo=slack">
  </a>

  <p align="center">
    Generates and verifies cryptographic proofs for SxT OLTP queries.
    <br />
    <a href="https://github.com/spaceandtimelabs/proofs"><strong>Explore the docs »</strong></a>
    <br />
    <br />
    <a href="https://space-and-time.atlassian.net/jira/software/c/projects/PROOF/boards/6/backlog">Report Bug</a>
    |
    <a href="https://space-and-time.atlassian.net/jira/software/c/projects/PROOF/boards/6/backlog">Request Feature</a>
  </p>
</div>

<!-- TABLE OF CONTENTS -->
<details>
  <summary>Table of Contents</summary>
  <ol>
    <li>
      <a href="#about-the-project">About The Project</a>
      <ul>
        <li><a href="#built-with">Built With</a></li>
      </ul>
    </li>
    <li>
      <a href="#getting-started">Getting Started</a>
      <ul>
        <li><a href="#prerequisites">Prerequisites</a></li>
        <li><a href="#installation">Installation</a></li>
      </ul>
    </li>
    <li><a href="#usage">Usage</a></li>
    <li><a href="#roadmap">Roadmap</a></li>
    <li><a href="#contributing">Contributing</a></li>
    <li><a href="#license">License</a></li>
    <li><a href="#contact">Contact</a></li>
    <li><a href="#acknowledgments">Acknowledgments</a></li>
  </ol>
</details>

<!-- ABOUT THE PROJECT -->
## About The Project

Generates and verifies cryptographic proofs for SxT OLTP queries.

<p align="right">(<a href="#top">back to top</a>)</p>

### Built With

<br />

[![Rust][Rust]][rust-url]

[![Semantic-Release][Semantic-Release]][semantic-release-url]

[![Conventional-Commits][Conventional-Commits]][conventional-commits-url]

[![CircleCI][CircleCI]][circleci-url]

<p align="right">(<a href="#top">back to top</a>)</p>

## Getting Started

To get a local copy up and running, follow these steps.

### Prerequisites

* Linux x86_64
* [Rust 1.62.0](https://www.rust-lang.org/tools/install)

### Installation

#### Using the Docker container:

If you have a Linux machine, a Docker installed, and a GPU available, run the following command:

```bash
bash ci/run_docker_with_gpu.sh
```

In case you don't have a GPU available, run:

```bash
bash ci/run_docker_with_cpu.sh
```

<p align="right">(<a href="#top">back to top</a>)</p>

<!-- USAGE EXAMPLES -->
## Usage

### Using the proofs Library in your project


### Tests

```bash
cargo test
```

### Documentation

```
$ cargo doc --no-deps --open
```

<p align="right">(<a href="#top">back to top</a>)</p>

<!-- ROADMAP -->
## Roadmap

### Milestone #1: Proof of Concept. See [here](https://github.com/spaceandtimelabs/proofs/blob/main/docs/proof-of-sql-alpha.md).
- [ ] Proofs Design
- [ ] Write designs for individual proof protocols. See [here](https://github.com/spaceandtimelabs/proofs/blob/main/docs/protocols/pips-sql-alpha.md).
- [ ] Create Framework for Datafusion Integration
    - [x] Integrate Physical Expressions
    - [x] Integrate Execution Plans
    - [ ] Integrate Aggregation
- [ ] Proofs Code
    - [ ] Write proofs for filter
    - [ ] Write proofs for aggregations
    - [ ] Write proofs for expressions
        - [ ] Implement safe integer expressions
        - [ ] Implement logical expressions
        - [ ] Implement comparison expressions
    - [ ] Implement String-type data types
### Milestone #2: Performance improvements
- [ ] Design
    - [ ] Write design docs the layout more expressive primitives.
    - [ ] Rewrite design docs with the performance improvements in mind.
- [ ] Write code for the extended primitives.
- [ ] Batching primitives
    - [ ] Redesign traits to support new scheme
    - [ ] Write batching methods for the primitives
- [ ] Convert proofs to new scheme
    - [ ] Rewrite proofs to implement the new primitive along with the batching scheme
### Milestone #3: Feature additions
- [ ] Design
    - [ ] Group By
    - [ ] Expressive Aggregations
    - [ ] Joins
- [ ] Implementation
    - [ ] Group By
    - [ ] Expressive Aggregations
    - [ ] Joins

See the [github open issues](https://github.com/spaceandtimelabs/proofs/issues) for a full list of proposed features (and known issues). Also, check our [JIRA board](https://space-and-time.atlassian.net/jira/software/c/projects/PROOF/boards/6/backlog).

<p align="right">(<a href="#top">back to top</a>)</p>

<!-- CONTRIBUTING -->
## Contributing

We are using semantic-release to automatically version our code. Alongside that, we adopted conventional commits to maintain our code history, which is used by semantic-release. Bear in mind that these two specify very precise rules that must be followed for the correct automatic release process. Please, check this [CONTRIBUTING](CONTRIBUTING.md) file for more information.

<p align="right">(<a href="#top">back to top</a>)</p>

## Continuous Integration (CI)

To allow semantic-release to publish to our GitHub release, you need to set up the following GitHub token in the CircleCI settings:

- **GH_TOKEN**: some Github user token with write privileges to the proofs repository

<p align="right">(<a href="#top">back to top</a>)</p>

<!-- CONTACT -->
## Contact

Ryan Burn - [@rnburn](https://github.com/rnburn) - ryan@spaceandtime.io

Ian Joiner - [@iajoiner](https://github.com/iajoiner) - ian.joiner@spaceandtime.io

Jay White - [@JayWhite2357](https://github.com/JayWhite2357) - jay@spaceandtime.io

Project Link: [https://github.com/spaceandtimelabs/proofs](https://github.com/spaceandtimelabs/proofs)

<p align="right">(<a href="#top">back to top</a>)</p>

[Semantic-Release]: https://img.shields.io/badge/semantic--release-6.0.3-blue
[semantic-release-url]: https://github.com/semantic-release/github

[Conventional-Commits]: https://img.shields.io/badge/conventional--commits-1.0.0-blue
[conventional-commits-url]: https://www.conventionalcommits.org/en/v1.0.0/

[CircleCI]: https://img.shields.io/badge/circleci-2.1-blue
[circleci-url]: https://circleci.com/

[Rust]: https://img.shields.io/badge/rust-1.62.0-blue
[rust-url]: https://www.rust-lang.org/
