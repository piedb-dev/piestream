/// All configurations available for a Decompressor.
#[derive(Clone, Debug)]
pub struct DecompressorConfig {
  /// The maximum number of numbers to decode at a time when streaming through
  /// the decompressor as an iterator.
  pub maxsize: usize,
}

impl Default for DecompressorConfig {
  fn default() -> Self {
    Self {
      maxsize: 100000,
    }
  }
}

impl DecompressorConfig {
    pub fn new(n: usize) -> Self {  
        self.maxsize = n; 
        self 
    }
}