// For the particularly repetitive case of passthrough ProvableExecutionPlans.
macro_rules! impl_provable_passthrough {
    ($provable:ty) => {
        impl Provable for $provable {
            fn get_proof(&self) -> ProofResult<Arc<DataFusionProof>> {
                (*self.proof.read().into_proof_result()?)
                    .clone()
                    .ok_or(ProofError::NoProofError)
            }
            fn set_proof(&self, proof: &Arc<DataFusionProof>) -> ProofResult<()> {
                let typed_proof: &TrivialProof = match &**proof {
                    ExecutionPlanProofEnumVariant(TrivialProofEnumVariant(p)) => p,
                    _ => return Err(ProofError::TypeError),
                };
                *self.proof.write().into_proof_result()? = Some(Arc::new(
                    ExecutionPlanProofEnumVariant(TrivialProofEnumVariant((*typed_proof).clone())),
                ));
                Ok(())
            }
            fn children(&self) -> &[Arc<dyn Provable>] {
                &self.provable_children[..]
            }
            fn run_create_proof(&self, transcript: &mut Transcript) -> ProofResult<()> {
                let input_table = Table::try_from(&self.input.output()?)?;
                let output_table = Table::try_from(&self.output()?)?;

                let c_in: Vec<Commitment> = match &*self.input.get_proof()? {
                    ExecutionPlanProofEnumVariant(exec_proof) => {
                        exec_proof.get_output_commitments()
                    }
                    _ => Err(ProofError::TypeError),
                }?;

                let proof = TrivialProof::prove(transcript, (input_table,), output_table, (c_in,));
                *self.proof.write().into_proof_result()? = Some(Arc::new(
                    ExecutionPlanProofEnumVariant(TrivialProofEnumVariant(proof)),
                ));
                Ok(())
            }
            fn run_verify(&self, transcript: &mut Transcript) -> ProofResult<()> {
                let proof = self.get_proof()?;
                match &*proof {
                    ExecutionPlanProofEnumVariant(TrivialProofEnumVariant(p)) => {
                        let input_proof: Arc<DataFusionProof> = self.input.get_proof()?;
                        let c_in: Vec<Commitment> = match &*input_proof {
                            ExecutionPlanProofEnumVariant(exec_proof) => {
                                exec_proof.get_output_commitments()
                            }
                            _ => Err(ProofError::TypeError),
                        }?;
                        p.verify(transcript, (c_in,))
                    }
                    _ => Err(ProofError::TypeError),
                }
            }
        }
    };
}
pub(crate) use impl_provable_passthrough;
