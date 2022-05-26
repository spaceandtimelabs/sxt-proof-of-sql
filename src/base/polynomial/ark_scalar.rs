use ark_ff::fields::{Fp256, MontBackend, MontConfig, MontFp};

#[derive(MontConfig)]
#[modulus = "7237005577332262213973186563042994240857116359379907606001950938285454250989"]
#[generator = "2"]
pub struct FqConfig;
pub type ArkScalar = Fp256<MontBackend<FqConfig, 4>>;
