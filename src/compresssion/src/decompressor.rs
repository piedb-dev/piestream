#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Decompressor<T> {
    rawdata: &Vec<T>,
}

impl<T> Decompressor<T> {
    pub fn new(rawdata: &Vec<T>) -> Self {}
    pub fn metadata(&mut self) -> Result<Metadata<T>> {}
    pub fn run(&mut self) -> Result<()> {}
}