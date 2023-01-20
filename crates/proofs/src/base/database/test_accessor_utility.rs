use super::TestAccessor;
use indexmap::IndexMap;
use rand::{
    distributions::{Distribution, Uniform},
    rngs::StdRng,
};

/// Specify what form a randomly generated TestAccessor can take
pub struct RandomTestAccessorDescriptor {
    pub min_rows: usize,
    pub max_rows: usize,
    pub min_value: i64,
    pub max_value: i64,
}

impl Default for RandomTestAccessorDescriptor {
    fn default() -> Self {
        Self {
            min_rows: 0,
            max_rows: 100,
            min_value: -5,
            max_value: 5,
        }
    }
}

/// Generate a TestAccessor with random data
pub fn make_random_test_accessor(
    rng: &mut StdRng,
    table: &str,
    cols: &[&str],
    descriptor: &RandomTestAccessorDescriptor,
) -> TestAccessor {
    let n = Uniform::new(descriptor.min_rows, descriptor.max_rows + 1).sample(rng);
    let mut data = IndexMap::new();
    let dist = Uniform::new(descriptor.min_value, descriptor.max_value + 1);
    for col in cols {
        let values = dist.sample_iter(&mut *rng).take(n).collect();
        data.insert(col.to_string(), values);
    }
    let mut res = TestAccessor::new();
    res.add_table(table, &data);
    res
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::database::accessor::MetadataAccessor;
    use crate::base::database::TableRef;
    use proofs_sql::ResourceId;
    use rand_core::SeedableRng;

    #[test]
    fn we_can_construct_a_random_test_accessor() {
        let descriptor = RandomTestAccessorDescriptor::default();
        let mut rng = StdRng::from_seed([0u8; 32]);
        let cols = ["a", "b"];
        let accessor1 = make_random_test_accessor(&mut rng, "abc", &cols, &descriptor);
        let accessor2 = make_random_test_accessor(&mut rng, "abc", &cols, &descriptor);
        let table_ref = TableRef::new(ResourceId::try_new("sxt", "abc").unwrap());
        assert_ne!(
            accessor1.get_length(&table_ref),
            accessor2.get_length(&table_ref)
        );
    }
}
