use params::{DefaultParams, RTreeParams};
use node::ParentNodeData;
use object::{PointDistance, RTreeObject};
use iterator::*;
use num_traits::Bounded;
use metrics::RTreeMetrics;
use locate::{LocateAll, LocateAllMut, LocateInEnvelope, LocateInEnvelopeMut};
use point::Point;

pub trait InsertionStrategy {
    fn insert<T, Params>(&mut RTree<T, Params>, t: T, metrics: &mut RTreeMetrics)
    where
        Params: RTreeParams,
        T: RTreeObject;
}

pub struct RTree<T, Params = DefaultParams>
where
    Params: RTreeParams,
    T: RTreeObject,
{
    root: ParentNodeData<T, Params>,
    size: usize,
    height: usize,
}

impl<T> RTree<T>
where
    T: RTreeObject,
{
    pub fn new() -> Self {
        Self::new_with_params()
    }
}

impl<T, Params> RTree<T, Params>
where
    Params: RTreeParams,
    T: RTreeObject,
{
    pub fn new_with_params() -> Self {
        RTree {
            root: ParentNodeData::new_root(),
            size: 0,
            height: 0,
        }
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn root(&self) -> &ParentNodeData<T, Params> {
        &self.root
    }

    pub fn root_mut(&mut self) -> &mut ParentNodeData<T, Params> {
        &mut self.root
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn set_height(&mut self, new_height: usize) {
        self.height = new_height;
    }

    #[cfg(not(feature = "debug"))]
    pub fn insert(&mut self, t: T) {
        Params::DefaultInsertionStrategy::insert(self, t, &mut RTreeMetrics {});
        self.size += 1;
    }

    #[cfg(feature = "debug")]
    pub fn insert(&mut self, t: T, metrics: &mut RTreeMetrics) {
        Params::DefaultInsertionStrategy::insert(self, t, metrics);
        self.size += 1;
    }

    pub fn iter(&self) -> RTreeIterator<T, Params> {
        RTreeIterator::new(self)
    }

    pub fn iter_mut(&mut self) -> RTreeIteratorMut<T, Params> {
        RTreeIteratorMut::new(self)
    }

    pub fn locate(&self, point: &T::Point) -> Option<&T> {
        self.locate_all(point).next()
    }

    pub fn locate_mut(&mut self, point: &T::Point) -> Option<&mut T> {
        self.locate_all_mut(point).next()
    }

    pub fn locate_all(&self, point: &T::Point) -> LocateAll<T, Params> {
        LocateAll::new(self, *point)
    }

    pub fn locate_all_mut(&mut self, point: &T::Point) -> LocateAllMut<T, Params> {
        LocateAllMut::new(self, *point)
    }

    pub fn locate_in_envelope(&self, envelope: &T::Envelope) -> LocateInEnvelope<T, Params> {
        LocateInEnvelope::new(self, *envelope)
    }

    pub fn locate_in_envelope_mut(
        &mut self,
        envelope: &T::Envelope,
    ) -> LocateInEnvelopeMut<T, Params> {
        LocateInEnvelopeMut::new(self, *envelope)
    }

    /*     checked_insert(&T) -> bool T: PartialEq
    checked_insert_mut(&T) -> Option<&mut T>
 */
}

impl<T, Params> RTree<T, Params>
where
    Params: RTreeParams,
    T: RTreeObject + PartialEq,
{
    pub fn contains(&self, t: &T) -> bool {
        self.locate_in_envelope(&t.envelope()).any(|e| e == t)
    }

    pub fn contains_mut(&mut self, t: &T) -> Option<&mut T> {
        self.locate_in_envelope_mut(&t.envelope()).find(|e| e == &t)
    }
}

impl<T, Params, P> RTree<T, Params>
where
    Params: RTreeParams,
    T: RTreeObject<Point = P> + PointDistance<Point = P>,
    P: Point,
{
    pub fn nearest_neighbor(&self, query_point: &P) -> Option<&T> {
        let mut max_value = Bounded::max_value();
        ::nearest_neighbor::nearest_neighbor(self.root(), query_point, &mut max_value)
    }
}

#[cfg(test)]
mod test {
    use typenum::{U1, U5, U7};
    use super::RTree;
    use params::{CustomParams, DefaultParams};
    use rstar::RStarInsertionStrategy;
    use generic_array::GenericArray;
    use testutils::create_random_points;

    #[test]
    fn test_create_rtree_with_parameters() {
        let tree: RTree<GenericArray<i32, U5>, DefaultParams> = RTree::new();
        let other_tree: RTree<
            GenericArray<f32, U7>,
            CustomParams<U5, U7, U1, RStarInsertionStrategy>,
        > = RTree::new_with_params();
        assert_eq!(tree.size(), 0);
        assert_eq!(other_tree.size(), 0);
    }

    #[test]
    fn test_insert_single() {
        let mut tree: RTree<_> = RTree::new();
        tree.insert([0.02f32, 0.4f32]);
        assert_eq!(tree.size(), 1);
        assert!(tree.contains(&[0.02, 0.4]));
        assert_eq!(tree.height(), 1);
        assert!(!tree.contains(&[0.3, 0.2]));
    }

    #[test]
    fn test_insert_many() {
        const NUM_POINTS: usize = 1000;
        let points = create_random_points(NUM_POINTS, [231, 22912, 399939, 922931]);
        let mut tree = RTree::new();
        for p in &points {
            tree.insert(*p);
        }
        assert_eq!(tree.size(), NUM_POINTS);
        for p in &points {
            assert!(tree.contains(p));
        }
    }
}
