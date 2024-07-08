pub trait Carrier {
    type Data;
    fn data(&self) -> &Self::Data;
}
