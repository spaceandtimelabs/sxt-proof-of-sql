use super::DynProofPlan;
use crate::{
    base::{
        database::{
            join_util::{
                apply_sort_merge_join_indexes, get_columns_of_table, get_sort_merge_join_indexes,
                ordered_set_union,
            },
            slice_operation::apply_slice_to_indexes,
            ColumnField, ColumnRef, OwnedTable, Table, TableEvaluation, TableOptions, TableRef,
        },
        map::{IndexMap, IndexSet},
        proof::ProofError,
        scalar::Scalar,
    },
    sql::{
        proof::{
            FinalRoundBuilder, FirstRoundBuilder, ProofPlan, ProverEvaluate,
            SumcheckSubpolynomialType, VerificationBuilder,
        },
        proof_gadgets::{
            final_round_evaluate_membership_check, final_round_evaluate_monotonic,
            first_round_evaluate_membership_check, first_round_evaluate_monotonic,
            verify_membership_check, verify_monotonic,
        },
    },
};
use alloc::{boxed::Box, vec, vec::Vec};
use bumpalo::{
    collections::{CollectIn, Vec as BumpVec},
    Bump,
};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use sqlparser::ast::Ident;

/// `ProofPlan` for queries of the form
/// ```ignore
///     <ProofPlan> INNER JOIN <ProofPlan>
///     ON col1 = col2
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SortMergeJoinExec {
    pub(super) left: Box<DynProofPlan>,
    pub(super) right: Box<DynProofPlan>,
    // `j_l` in the protocol
    pub(super) left_join_column_indexes: Vec<usize>,
    // `j_r` in the protocol
    pub(super) right_join_column_indexes: Vec<usize>,
    pub(super) result_idents: Vec<Ident>,
}

impl SortMergeJoinExec {
    /// Create a new `SortMergeJoinExec` with the given left and right plans
    ///
    /// # Panics
    /// Panics if one of the following conditions is met:
    /// - The join column index is out of bounds
    /// - The number of join columns is different
    /// - The number of result idents is different from the expected number of columns
    pub fn new(
        left: Box<DynProofPlan>,
        right: Box<DynProofPlan>,
        left_join_column_indexes: Vec<usize>,
        right_join_column_indexes: Vec<usize>,
        result_idents: Vec<Ident>,
    ) -> Self {
        let num_columns_left = left.get_column_result_fields().len();
        let num_columns_right = right.get_column_result_fields().len();
        let max_left_join_column_index = left_join_column_indexes.iter().max().unwrap_or(&0);
        let max_right_join_column_index = right_join_column_indexes.iter().max().unwrap_or(&0);
        if *max_left_join_column_index >= num_columns_left
            || *max_right_join_column_index >= num_columns_right
        {
            panic!("Join column index out of bounds");
        }
        let num_join_columns = left_join_column_indexes.len();
        assert!(
            (num_join_columns == right_join_column_indexes.len()),
            "Join columns should have the same number of columns"
        );
        assert!(
            (result_idents.len() == num_columns_left + num_columns_right - num_join_columns),
            "The amount of result idents should be the same as the expected number of columns"
        );
        Self {
            left,
            right,
            left_join_column_indexes,
            right_join_column_indexes,
            result_idents,
        }
    }
}

impl ProofPlan for SortMergeJoinExec
where
    SortMergeJoinExec: ProverEvaluate,
{
    #[allow(clippy::too_many_lines, clippy::similar_names)]
    fn verifier_evaluate<S: Scalar, B: VerificationBuilder<S>>(
        &self,
        builder: &mut B,
        accessor: &IndexMap<ColumnRef, S>,
        _result: Option<&OwnedTable<S>>,
        one_eval_map: &IndexMap<TableRef, S>,
    ) -> Result<TableEvaluation<S>, ProofError> {
        // 1. columns
        // TODO: Make sure `GroupByExec` as self.input is supported
        let left_eval = self
            .left
            .verifier_evaluate(builder, accessor, None, one_eval_map)?;
        let right_eval = self
            .right
            .verifier_evaluate(builder, accessor, None, one_eval_map)?;
        // 2. One evals and rho evals
        let left_chi_eval = left_eval.one_eval();
        let right_chi_eval = right_eval.one_eval();
        let res_chi_eval = builder.try_consume_one_evaluation()?;
        let u_chi_eval = builder.try_consume_one_evaluation()?;
        let left_rho_eval = builder.try_consume_rho_evaluation()?;
        let right_rho_eval = builder.try_consume_rho_evaluation()?;
        // 3. alpha, beta
        let alpha = builder.try_consume_post_result_challenge()?;
        let beta = builder.try_consume_post_result_challenge()?;
        // 4. column evals
        let left_column_evals = left_eval.column_evals();
        let right_column_evals = right_eval.column_evals();
        let num_columns_left = left_column_evals.len();
        let num_columns_right = right_column_evals.len();
        let left_hat_column_evals = left_column_evals
            .iter()
            .chain(core::iter::once(&left_rho_eval))
            .copied()
            .collect::<Vec<_>>();
        let right_hat_column_evals = right_column_evals
            .iter()
            .chain(core::iter::once(&right_rho_eval))
            .copied()
            .collect::<Vec<_>>();
        let num_columns_u = self.left_join_column_indexes.len();
        if num_columns_u != 1 {
            return Err(ProofError::VerificationError {
                error: "Join on multiple columns not supported yet",
            });
        }
        let num_columns_res_hat = num_columns_left + num_columns_right - num_columns_u + 2;
        // `\hat{J}` in the protocol
        let res_hat_column_evals =
            builder.try_consume_final_round_mle_evaluations(num_columns_res_hat)?;
        // 5. First round MLE evaluations: `i` and `U`
        //TODO: Make it possible for `U` to have multiple columns
        let rho_bar_left_eval = res_hat_column_evals[num_columns_left];
        let rho_bar_right_eval = res_hat_column_evals[num_columns_res_hat - 1];
        let i_eval: S = itertools::repeat_n(S::TWO, 64_usize).product::<S>() * rho_bar_left_eval
            + rho_bar_right_eval;
        let u_column_eval = builder.try_consume_first_round_mle_evaluation()?;
        // 6. Membership checks
        let hat_left_column_indexes = self
            .left_join_column_indexes
            .iter()
            .copied()
            .chain((0..=num_columns_left).filter(|i| !self.left_join_column_indexes.contains(i)))
            .collect::<Vec<_>>();
        let hat_right_column_indexes = self
            .right_join_column_indexes
            .iter()
            .copied()
            .chain((0..=num_columns_right).filter(|i| !self.right_join_column_indexes.contains(i)))
            .collect::<Vec<_>>();
        let hat_left_column_evals =
            apply_slice_to_indexes(&left_hat_column_evals, &hat_left_column_indexes)
                .expect("Indexes can not be out of bounds");
        let hat_right_column_evals =
            apply_slice_to_indexes(&right_hat_column_evals, &hat_right_column_indexes)
                .expect("Indexes can not be out of bounds");
        let res_left_column_indexes = (0..=num_columns_left).collect::<Vec<_>>();
        let res_right_column_indexes = (0..num_columns_u)
            .chain(num_columns_left + 1..num_columns_res_hat)
            .collect::<Vec<_>>();
        let res_left_column_evals =
            apply_slice_to_indexes(&res_hat_column_evals, &res_left_column_indexes)
                .expect("Indexes can not be out of bounds");
        let res_right_column_evals =
            apply_slice_to_indexes(&res_hat_column_evals, &res_right_column_indexes)
                .expect("Indexes can not be out of bounds");
        verify_membership_check(
            builder,
            alpha,
            beta,
            left_chi_eval,
            res_chi_eval,
            &hat_left_column_evals,
            &res_left_column_evals,
        )?;
        verify_membership_check(
            builder,
            alpha,
            beta,
            right_chi_eval,
            res_chi_eval,
            &hat_right_column_evals,
            &res_right_column_evals,
        )?;
        let left_join_column_evals =
            apply_slice_to_indexes(&left_hat_column_evals, &self.left_join_column_indexes)
                .expect("Indexes can not be out of bounds");
        let right_join_column_evals =
            apply_slice_to_indexes(&right_hat_column_evals, &self.right_join_column_indexes)
                .expect("Indexes can not be out of bounds");
        //TODO: Relax to allow multiple columns
        if left_join_column_evals.len() != 1 || right_join_column_evals.len() != 1 {
            return Err(ProofError::VerificationError {
                error: "Left and right join columns should have exactly one column",
            });
        }
        let w_l_eval = verify_membership_check(
            builder,
            alpha,
            beta,
            u_chi_eval,
            left_chi_eval,
            &[u_column_eval],
            &left_join_column_evals,
        )?;
        let w_r_eval = verify_membership_check(
            builder,
            alpha,
            beta,
            u_chi_eval,
            right_chi_eval,
            &[u_column_eval],
            &right_join_column_evals,
        )?;
        // 7. Monotonicity checks
        verify_monotonic::<S, true, true, _>(builder, alpha, beta, i_eval, res_chi_eval)?;
        verify_monotonic::<S, true, true, _>(builder, alpha, beta, u_column_eval, u_chi_eval)?;
        // 8. Prove that sum w_l * w_r = chi_m
        // sum w_l * w_r - chi_m = 0
        builder.try_produce_sumcheck_subpolynomial_evaluation(
            SumcheckSubpolynomialType::ZeroSum,
            w_l_eval * w_r_eval - res_chi_eval,
            2,
        )?;
        // 9. Return the result
        // Drop the two rho columns of `\hat{J}` to get `J`
        let res_column_indexes = (0..num_columns_left)
            .chain(num_columns_left + 1..num_columns_left + 1 + num_columns_right - num_columns_u)
            .collect::<Vec<_>>();
        let res_column_evals = apply_slice_to_indexes(&res_hat_column_evals, &res_column_indexes)
            .expect("Indexes can not be out of bounds");
        Ok(TableEvaluation::new(res_column_evals, res_chi_eval))
    }

    fn get_column_result_fields(&self) -> Vec<ColumnField> {
        let left_other_column_indexes = (0..self.left.get_column_result_fields().len())
            .filter(|i| !self.left_join_column_indexes.contains(i))
            .collect::<Vec<_>>();
        let right_other_column_indexes = (0..self.right.get_column_result_fields().len())
            .filter(|i| !self.right_join_column_indexes.contains(i))
            .collect::<Vec<_>>();
        let left_join_column_fields = apply_slice_to_indexes(
            &self.left.get_column_result_fields(),
            &self.left_join_column_indexes,
        )
        .expect("Indexes can not be out of bounds");
        let left_other_column_fields = apply_slice_to_indexes(
            &self.left.get_column_result_fields(),
            &left_other_column_indexes,
        )
        .expect("Indexes can not be out of bounds");
        let right_other_column_fields = apply_slice_to_indexes(
            &self.right.get_column_result_fields(),
            &right_other_column_indexes,
        )
        .expect("Indexes can not be out of bounds");
        let column_types = left_join_column_fields
            .iter()
            .chain(left_other_column_fields.iter())
            .chain(right_other_column_fields.iter())
            .map(ColumnField::data_type)
            .collect::<Vec<_>>();
        self.result_idents
            .iter()
            .zip_eq(column_types)
            .map(|(ident, column_type)| ColumnField::new(ident.clone(), column_type))
            .collect()
    }

    fn get_column_references(&self) -> IndexSet<ColumnRef> {
        self.left
            .get_column_references()
            .into_iter()
            .chain(self.right.get_column_references())
            .collect()
    }

    fn get_table_references(&self) -> IndexSet<TableRef> {
        self.left
            .get_table_references()
            .into_iter()
            .chain(self.right.get_table_references())
            .collect()
    }
}

impl ProverEvaluate for SortMergeJoinExec {
    #[tracing::instrument(
        name = "SortMergeJoinExec::first_round_evaluate",
        level = "debug",
        skip_all
    )]
    fn first_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FirstRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table_map: &IndexMap<TableRef, Table<'a, S>>,
    ) -> Table<'a, S> {
        let left = self.left.first_round_evaluate(builder, alloc, table_map);
        let right = self.right.first_round_evaluate(builder, alloc, table_map);
        let num_rows_left = left.num_rows();
        let num_rows_right = right.num_rows();
        let num_columns_left = left.num_columns();
        let num_columns_right = right.num_columns();
        let left_hat = left.add_rho_column(alloc);
        let right_hat = right.add_rho_column(alloc);
        let c_l = get_columns_of_table(&left_hat, &self.left_join_column_indexes)
            .expect("Indexes can not be out of bounds");
        let c_r = get_columns_of_table(&right_hat, &self.right_join_column_indexes)
            .expect("Indexes can not be out of bounds");
        // 1. Conduct the join
        let (left_row_indexes, right_row_indexes): (Vec<usize>, Vec<usize>) =
            get_sort_merge_join_indexes(&c_l, &c_r, num_rows_left, num_rows_right)
                .iter()
                .copied()
                .unzip();
        // `\hat{J}` in the protocol
        let res_hat = apply_sort_merge_join_indexes(
            &left_hat,
            &right_hat,
            &self.left_join_column_indexes,
            &self.right_join_column_indexes,
            &left_row_indexes,
            &right_row_indexes,
            alloc,
        )
        .expect("Can not do sort merge join");
        let num_rows_res = left_row_indexes.len();
        // 2. Get and commit the strictly increasing columns, `U`
        // ordered set union `U`
        let u = ordered_set_union(&c_l, &c_r, alloc).unwrap();
        let num_columns_u = u.len();
        assert!(
            (num_columns_u == 1),
            "Join on multiple columns not supported yet"
        );
        let u_0 = u[0].to_scalar_with_scaling(0);
        let num_rows_u = u[0].len();
        let alloc_u_0 = alloc.alloc_slice_copy(u_0.as_slice());
        builder.produce_intermediate_mle(alloc_u_0 as &[_]);
        // 3. One eval and rho eval
        builder.produce_one_evaluation_length(num_rows_res);
        builder.produce_one_evaluation_length(num_rows_u);
        builder.produce_rho_evaluation_length(num_rows_left);
        builder.produce_rho_evaluation_length(num_rows_right);
        // 4. Membership checks
        let hat_left_column_indexes = self
            .left_join_column_indexes
            .iter()
            .copied()
            .chain((0..=num_columns_left).filter(|i| !self.left_join_column_indexes.contains(i)))
            .collect::<Vec<_>>();
        let hat_right_column_indexes = self
            .right_join_column_indexes
            .iter()
            .copied()
            .chain((0..=num_columns_right).filter(|i| !self.right_join_column_indexes.contains(i)))
            .collect::<Vec<_>>();
        let hat_left_columns = get_columns_of_table(&left_hat, &hat_left_column_indexes)
            .expect("Indexes can not be out of bounds");
        let hat_right_columns = get_columns_of_table(&right_hat, &hat_right_column_indexes)
            .expect("Indexes can not be out of bounds");
        // `J_l` in the protocol
        let res_left_columns = res_hat[0..=num_columns_left].to_vec();
        // `J_r` in the protocol
        let res_right_columns: Vec<_> = res_hat[0..num_columns_u]
            .iter()
            .chain(&res_hat[num_columns_left + 1..])
            .copied()
            .collect();
        first_round_evaluate_membership_check(builder, alloc, &hat_left_columns, &res_left_columns);
        first_round_evaluate_membership_check(
            builder,
            alloc,
            &hat_right_columns,
            &res_right_columns,
        );
        first_round_evaluate_membership_check(builder, alloc, &u, &c_l);
        first_round_evaluate_membership_check(builder, alloc, &u, &c_r);
        // 5. Monotonicity checks
        first_round_evaluate_monotonic(builder, num_rows_res);
        first_round_evaluate_monotonic(builder, num_rows_u);
        // 6. Request post-result challenges
        builder.request_post_result_challenges(2);
        // 7. Return join result
        // Drop the two rho columns of `\hat{J}` to get `J`
        let res_column_indexes = (0..num_columns_left)
            .chain(num_columns_left + 1..num_columns_left + 1 + num_columns_right - num_columns_u)
            .collect::<Vec<_>>();
        let res_columns = apply_slice_to_indexes(&res_hat, &res_column_indexes)
            .expect("Indexes can not be out of bounds");
        let tab = Table::try_from_iter_with_options(
            self.result_idents.iter().cloned().zip_eq(res_columns),
            TableOptions::new(Some(num_rows_res)),
        )
        .expect("Can not create table");
        tab
    }

    #[tracing::instrument(
        name = "SortMergeJoinExec::final_round_evaluate",
        level = "debug",
        skip_all
    )]
    #[allow(unused_variables)]
    fn final_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table_map: &IndexMap<TableRef, Table<'a, S>>,
    ) -> Table<'a, S> {
        let left = self.left.final_round_evaluate(builder, alloc, table_map);
        let right = self.right.final_round_evaluate(builder, alloc, table_map);
        let num_rows_left = left.num_rows();
        let num_rows_right = right.num_rows();
        let num_columns_left = left.num_columns();
        let num_columns_right = right.num_columns();

        let chi_m_l = alloc.alloc_slice_fill_copy(num_rows_left, true);
        let chi_m_r = alloc.alloc_slice_fill_copy(num_rows_right, true);

        let left_hat = left.add_rho_column(alloc);
        let right_hat = right.add_rho_column(alloc);

        let c_l = get_columns_of_table(&left_hat, &self.left_join_column_indexes)
            .expect("Indexes can not be out of bounds");
        let c_r = get_columns_of_table(&right_hat, &self.right_join_column_indexes)
            .expect("Indexes can not be out of bounds");

        // 1. Conduct the join
        let (left_row_indexes, right_row_indexes): (Vec<usize>, Vec<usize>) =
            get_sort_merge_join_indexes(&c_l, &c_r, num_rows_left, num_rows_right)
                .iter()
                .copied()
                .unzip();

        // Instead of storing the join result in a local `Vec`, we copy it into bump-allocated memory
        // so it will outlive this scope (matching the `'a` lifetime) and avoid borrow issues.
        let raw_res_hat = apply_sort_merge_join_indexes(
            &left_hat,
            &right_hat,
            &self.left_join_column_indexes,
            &self.right_join_column_indexes,
            &left_row_indexes,
            &right_row_indexes,
            alloc,
        )
        .expect("Can not do sort merge join");
        // Store in bump, `\hat{J}` in the protocol
        let res_hat = alloc.alloc_slice_copy(raw_res_hat.as_slice());

        let num_rows_res = left_row_indexes.len();
        let res_ones = alloc.alloc_slice_fill_copy(num_rows_res, true);

        // 2. Get the strictly increasing columns, `i` and `u`
        // i = left_row_index * 2^64 + right_row_index
        // which is strictly increasing
        let i = left_row_indexes
            .iter()
            .zip_eq(right_row_indexes.iter())
            .map(|(l, r)| S::from(*l as u64) * S::TWO_POW_64 + S::from(*r as u64))
            .collect::<Vec<_>>();
        let alloc_i = alloc.alloc_slice_copy(i.as_slice());

        // ordered set union `U`
        let u = ordered_set_union(&c_l, &c_r, alloc).unwrap();
        let num_columns_u = u.len();
        assert!(
            (num_columns_u == 1),
            "Join on multiple columns not supported yet"
        );
        let u_0 = u[0].to_scalar_with_scaling(0);
        let num_rows_u = u[0].len();
        let alloc_u_0 = alloc.alloc_slice_copy(u_0.as_slice());
        let u_ones = alloc.alloc_slice_fill_copy(num_rows_u, true);
        let alloc_u_0 = alloc.alloc_slice_copy(u_0.as_slice());

        // 3. Get post-result challenges
        let alpha = builder.consume_post_result_challenge();
        let beta = builder.consume_post_result_challenge();

        // 4. Produce MLEs for `res_hat`
        // We can reference `res_hat` safely because it's bump-allocated.
        let alloc_res_hat = res_hat.iter().collect_in::<BumpVec<_>>(alloc);
        for column in &alloc_res_hat {
            builder.produce_intermediate_mle(*column);
        }

        // 5. Membership checks
        let hat_left_column_indexes = self
            .left_join_column_indexes
            .iter()
            .copied()
            .chain((0..=num_columns_left).filter(|i| !self.left_join_column_indexes.contains(i)))
            .collect::<Vec<_>>();
        let hat_right_column_indexes = self
            .right_join_column_indexes
            .iter()
            .copied()
            .chain((0..=num_columns_right).filter(|i| !self.right_join_column_indexes.contains(i)))
            .collect::<Vec<_>>();

        let hat_left_columns = get_columns_of_table(&left_hat, &hat_left_column_indexes)
            .expect("Indexes can not be out of bounds");
        let hat_right_columns = get_columns_of_table(&right_hat, &hat_right_column_indexes)
            .expect("Indexes can not be out of bounds");

        let res_left_columns = res_hat[0..=num_columns_left].to_vec();
        let res_right_columns: Vec<_> = res_hat[0..num_columns_u] // rho col is right after left columns
            .iter()
            .chain(&res_hat[num_columns_left + 1..])
            .copied()
            .collect();

        final_round_evaluate_membership_check(
            builder,
            alloc,
            alpha,
            beta,
            chi_m_l,
            res_ones,
            &hat_left_columns,
            &res_left_columns,
        );
        final_round_evaluate_membership_check(
            builder,
            alloc,
            alpha,
            beta,
            chi_m_r,
            res_ones,
            &hat_right_columns,
            &res_right_columns,
        );
        let w_l = final_round_evaluate_membership_check(
            builder, alloc, alpha, beta, u_ones, chi_m_l, &u, &c_l,
        );
        let w_r = final_round_evaluate_membership_check(
            builder, alloc, alpha, beta, u_ones, chi_m_r, &u, &c_r,
        );

        // 6. Monotonicity checks
        final_round_evaluate_monotonic::<S, true, true>(builder, alloc, alpha, beta, alloc_i);
        final_round_evaluate_monotonic::<S, true, true>(builder, alloc, alpha, beta, alloc_u_0);

        // 7. Prove that sum w_l * w_r = chi_m
        // sum w_l * w_r - chi_m = 0
        builder.produce_sumcheck_subpolynomial(
            SumcheckSubpolynomialType::ZeroSum,
            vec![
                (S::one(), vec![Box::new(w_l as &[_]), Box::new(w_r as &[_])]),
                (-S::one(), vec![Box::new(res_ones as &[_])]),
            ],
        );

        // 8. Return join result
        // Drop the two rho columns of `\hat{J}` to get `J`
        let res_column_indexes = (0..num_columns_left)
            .chain(num_columns_left + 1..num_columns_left + 1 + num_columns_right - num_columns_u)
            .collect::<Vec<_>>();
        let res_columns = apply_slice_to_indexes(res_hat, &res_column_indexes)
            .expect("Indexes can not be out of bounds");

        Table::try_from_iter_with_options(
            self.result_idents.iter().cloned().zip_eq(res_columns),
            TableOptions::new(Some(num_rows_res)),
        )
        .expect("Can not create table")
    }
}
