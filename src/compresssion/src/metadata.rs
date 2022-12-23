#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct Metadata<T> {
  /// The count of numbers in the chunk.
  pub n: usize,
  /// The compressed byte length of the body that immediately follow this chunk metadata section.
  pub compressed_body_size: usize,
  /// real meta data (including auxillary data, prefixes, dictionaries, etc).
  data: Vec<T>,
}