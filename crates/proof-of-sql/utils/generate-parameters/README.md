# Space and Time ParamGen

A simple tool to generate the Space and Time public network parameters.

## 📑 Table of Contents

- [🛠️ Installation](#installation)
- [📚 Background](#background)
- [🚀 Usage](#usage)
- [📧 Contact](#contact)
- [📚 Additional Resources](#additional-resources)

## <a name="installation"></a>🛠️ Installation

Install rust:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## <a name="background"></a>📚 Background

### What are public parameters?

There are a wide variety of zero-knowledge proof and argument systems, all offering different performance characterists. The classic example is the Groth16 argument, which establishes a trusted setup (known formally as a common reference string (CRS) or structured reference string (SRS)) to be shared among participants in the network. This setup is structured in such a way that allows arguments of valid computation to be produced with very small sizes. In the case of Groth16, this can be as low as a few group elements or a couple hundred bytes, which is the perfect size to store on a blockchain.

The Space and Time network makes use of a few different argument systems. The Dory polynomial commitment scheme (PCS) is is a SNARK which requires a setup to be established between the proving and verifying parties. The Dory PCS is chosen because it is ammenable to forming proofs and arguments over matrices, which is perfect for the Proof-Of-SQL case, since databases and tables are essentially matrices. The Dory setup process is unique in that it is *transparent*, meaning there is no toxic waste or secret values to forget once the setup is complete. The setup is initialized with an arbitrary random string which establishes common parameters. We choose the random string "SpaceAndTime" for our setup. This string is a "nothing-up-my-sleeve" number, meaning it is easily auditable and has no hidden structure that can be exploited to generate false proofs or compromise the integrity of the system.

The Space and Time implementation of the Dory PCS is non-zero knowledge and does not explicty blind the inputs used in the argument of correct sql execution. This yields a leaner implementation and slightly better performance. We may add zero-knowledge blinding in the future, but for now it is not necessary for Proof-Of-SQL to function correctly.

This tool generates the public setups for either the prover or verifier. Both setups are parameterized over a value *nu*, which helps establish the maximum dimension of the table that can be argued against. The prover and the verifer both posses a slightly different setup. The verifier setup is relatively cheap to compute and scales linearly for large nu/table sizes. The prover setup is larger and has a higher cost to compute. We provide pre-computed setups that can easily be downloaded and used with the SxT network in order to skip the expensive generation process, but this repo contains a tool to generate the parameters at your option.

## <a name="usage"></a>🚀 Usage

Ensure that you have rust installed. Then, clone this repo and simply run the following:

```bash
cargo run --release
```

This generates the setups for both the prover and verifer as two seperate tar.gz files with a default nu value of 14. It saves these parameters at the head of this repo as tar.gz archives.

You can also just generate the prover setup only:

```bash
cargo run --release -- --mode  p
```

Or just the verifier:

```bash
cargo run --release -- --mode  v
```

You can also specify a value of nu if needed:

```bash
cargo run --release -- --mode  pv --nu 4
```

## <a name="additional-resources"></a>📚 Additional Resources

- [Dory: Efficient, Transparent arguments for Generalised Inner Products and Polynomial Commitments](https://eprint.iacr.org/2020/1274)
- [Groth16](https://eprint.iacr.org/2016/260.pdf)
- [Nothing-up-my-sleeve number](https://en.wikipedia.org/wiki/Nothing-up-my-sleeve_number)
