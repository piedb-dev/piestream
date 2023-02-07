pub enum CompressionAlgorithm {
    Zstd,
    Qtile,
    Snappy,
    ZigZag,
    Lz4,
    Simple8b,
    Varint,
    Clp,
    Delta,
    Delta2,
    DeltaOfDelta,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Compressor<T> {
    rawdata: &Vec<T>,
    algo: CompressionAlgorithms,
    level: uint8,
}

impl<T> Compressor<T> {
    pub fn new(rawdata: &Vec<T>) -> Self {}
    pub fn config(&mut self, algo: CompressionAlgorithms, level: uint8) -> Result<Self> {}
    pub fn metadata(&mut self) -> Result<Metadata<T>> {}
    pub fn run(&mut self) -> Result<()> {}
}


