#[derive(Debug, Clone, PartialEq)]
pub struct WeightedChoice<T> {
    pub value: T,
    pub weight: f32,
}
