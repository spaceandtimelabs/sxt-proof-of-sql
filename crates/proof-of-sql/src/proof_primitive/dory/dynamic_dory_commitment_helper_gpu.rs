use super::{
    blitzar_metadata_table::{create_blitzar_metadata_tables, signed_commits},
    pairings, DynamicDoryCommitment, G1Affine, ProverSetup,
};
use crate::{
    base::{commitment::CommittableColumn, if_rayon, slice_ops::slice_cast},
    proof_primitive::dynamic_matrix_utils::matrix_structure::row_and_column_from_index,
};
use blitzar::compute::ElementP2;
#[cfg(feature = "rayon")]
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use sysinfo::{System, SystemExt};
use tracing::{debug, span, Level};

fn log_start_memory_usage() {
    log_memory_usage("Start");
}

fn log_end_memory_usage() {
    log_memory_usage("End");
}

#[allow(clippy::cast_precision_loss)]
fn log_memory_usage(name: &str) {
    if tracing::level_enabled!(Level::DEBUG) {
        let mut system = System::new_all();
        system.refresh_memory();

        let available_memory = system.available_memory() as f64 / 1024.0;
        let used_memory = system.used_memory() as f64 / 1024.0;
        let percentage_memory_used = (used_memory / (used_memory + available_memory)) * 100.0;

        tracing::Span::current().record("available_memory", available_memory);
        tracing::Span::current().record("used_memory", used_memory);
        tracing::Span::current().record("percentage_memory_used", percentage_memory_used);

        debug!(
            "{} Available memory: {:.2} MB, Used memory: {:.2} MB, Percentage memory used: {:.2}%",
            name, available_memory, used_memory, percentage_memory_used
        );
    }
}

/// Computes the dynamic Dory commitment using the GPU implementation of the `vlen_msm` algorithm.
///
/// # Arguments
///
/// * `committable_columns` - A reference to the committable columns.
/// * `offset` - The offset to the data.
/// * `setup` - A reference to the prover setup.
///
/// # Returns
///
/// A vector containing the dynamic Dory commitments.
///
/// # Panics
///
/// Panics if the number of sub commits is not a multiple of the number of committable columns.
#[tracing::instrument(
    name = "compute_dynamic_dory_commitments (gpu)",
    level = "debug",
    skip_all,
    fields(available_memory, used_memory, percentage_memory_used)
)]
pub(super) fn compute_dynamic_dory_commitments(
    committable_columns: &[CommittableColumn],
    offset: usize,
    setup: &ProverSetup,
) -> Vec<DynamicDoryCommitment> {
    if committable_columns.is_empty() {
        return vec![];
    }

    let Gamma_2 = setup.Gamma_2.last().unwrap();
    let (gamma_2_offset, _) = row_and_column_from_index(offset);

    // Get metadata tables for Blitzar's vlen_msm algorithm.
    let (blitzar_output_bit_table, blitzar_output_length_table, blitzar_scalars) =
        create_blitzar_metadata_tables(committable_columns, offset);

    // Initialize sub commits.
    let mut blitzar_sub_commits =
        vec![ElementP2::<ark_bls12_381::g1::Config>::default(); blitzar_output_bit_table.len()];

    // Get sub commits from Blitzar's vlen_msm algorithm.
    setup.blitzar_vlen_msm(
        &mut blitzar_sub_commits,
        &blitzar_output_bit_table,
        &blitzar_output_length_table,
        blitzar_scalars.as_slice(),
    );

    // Modify the sub commits to include the signed offset.
    let all_sub_commits: Vec<G1Affine> = slice_cast(&blitzar_sub_commits);
    let signed_sub_commits = signed_commits(&all_sub_commits, committable_columns);
    assert!(
        signed_sub_commits.len() % committable_columns.len() == 0,
        "Invalid number of sub commits"
    );
    let num_commits = signed_sub_commits.len() / committable_columns.len();

    // Calculate the dynamic Dory commitments.
    let span = span!(Level::INFO, "multi_pairing").entered();
    let ddc: Vec<DynamicDoryCommitment> = signed_sub_commits
        .is_empty()
        .then_some(vec![
            DynamicDoryCommitment::default();
            committable_columns.len()
        ])
        .unwrap_or_else(|| {
            if_rayon!(
                (0..committable_columns.len()).into_par_iter(),
                (0..committable_columns.len())
            )
            .map(|i| {
                let sub_slice = signed_sub_commits[i..]
                    .iter()
                    .step_by(committable_columns.len())
                    .take(num_commits);
                DynamicDoryCommitment(pairings::multi_pairing(
                    sub_slice,
                    &Gamma_2[gamma_2_offset..gamma_2_offset + num_commits],
                ))
            })
            .collect()
        });
    span.exit();

    log_end_memory_usage();

    ddc
}
