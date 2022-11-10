//! visualizations for various steps of the world generation process
struct ShoresAndWatershed<'a, 'b, 'c> {
    erode_map: super::ErodeMap<'a>,
    shores_map: &'b [usize],
    watershed_map: &'c [bool],
}
