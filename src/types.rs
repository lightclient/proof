use std::marker::PhantomData;
use typenum::Unsigned;

#[derive(Debug, Default)]
pub struct VariableList<T, N: Unsigned> {
    _phantom_type: PhantomData<T>,
    _phantom_count: PhantomData<N>,
}

#[derive(Debug, Default)]
pub struct FixedVector<T, N: Unsigned> {
    _phantom_type: PhantomData<T>,
    _phantom_count: PhantomData<N>,
}
